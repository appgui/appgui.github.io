# Package Release v1.0.0 - Manual Setup Requirements

This document outlines the non-automatable actions required to prepare the GitHub Actions workflow for package release v1.0.0 across Linux, Windows, macOS, and Android platforms.

## Overview

The automated workflow is configured in `.github/workflows/build.yml` and triggers on:

- **Tag Push**: When a tag matching `v*` (e.g., `v1.0.0`) is pushed to GitHub
- **Repository Dispatch**: Manual trigger with type `build-release` (for GitLab integration)
  The workflow builds binary packages for:
- Linux (x86_64, ARM64)
- Windows (x86_64)
- macOS (ARM64 / Apple Silicon)
- Android (ARM64 APK)

## Required Manual Setup Actions

### 1. GitHub Repository Settings

**Location**: Settings → Secrets and variables → Actions

#### Required Secrets (Optional - for code signing):

- [ ] `WINDOWS_CERTIFICATE_COMPANY` - Company name for Windows Authenticode signing
- [ ] `WINDOWS_CERTIFICATE_BASE64` - Base64-encoded PFX/PKCS12 certificate for Windows binary signing
- [ ] `WINDOWS_CERTIFICATE_PASSWORD` - Password for the Windows certificate
- [ ] `ANDROID_KEYSTORE_PASSWORD` - Password for Android APK signing keystore
  - Note: Android keystore is automatically generated if password is provided
  - Used for signing APK releases for distribution on app stores

### 2. Android Release Keystore Setup

**Current Status**: Semi-automatic with manual credential injection
**Steps**:

1. Generate or obtain an Android signing keystore for production:
   ```bash
   keytool -genkey -v -keystore release.keystore \
     -alias release \
     -keyalg RSA \
     -keysize 2048 \
     -validity 10000 \
     -storepass <password> \
     -keypass <password> \
     -dname "CN=Your Name, OU=Your Org, O=Your Company, L=City, ST=State, C=Country"
   ```
2. Store the keystore password in GitHub Secrets as `ANDROID_KEYSTORE_PASSWORD`
   - The workflow will use this to configure APK signing during CI/CD builds
   - If not configured, Android builds will be skipped (gracefully fails)
3. **Important**: Keep the keystore file itself in a secure location (NOT in the repository):
   - This is generated locally during the build process
   - For production releases, use a proper CI/CD secret management system
   - Never commit keystore files to Git

### 3. Windows Code Signing Setup (Optional)

**Current Status**: Configured but not actively used in build steps
**To Enable Windows Binary Signing**:

1. Obtain or create a code signing certificate:
   - Option A: Purchase from Certificate Authority (DigiCert, Sectigo, etc.)
   - Option B: Create self-signed certificate for internal distribution only
   - Option C: Use your organization's existing PKI infrastructure
2. Convert certificate to PFX/PKCS12 format if needed:
   ```bash
   # From PEM format:
   openssl pkcs12 -export -out certificate.pfx \
     -inkey private.key \
     -in certificate.crt \
     -certfile ca-chain.crt
   ```
3. Encode the PFX file to Base64:
   ```bash
   base64 -i certificate.pfx -o certificate.b64
   ```
4. Add GitHub Secrets:
   - `WINDOWS_CERTIFICATE_BASE64`: Content of certificate.b64
   - `WINDOWS_CERTIFICATE_PASSWORD`: Certificate password
   - `WINDOWS_CERTIFICATE_COMPANY`: Your company name for Authenticode
5. Update the Windows build step in `.github/workflows/build.yml` to use the certificate:
   ```yaml
   - name: Import and sign Windows binary
     if: matrix.platform == 'windows/amd64'
     run: |
       # Decode certificate
       echo "${{ secrets.WINDOWS_CERTIFICATE_BASE64 }}" | base64 -d > cert.pfx
       # Sign binary (requires signtool.exe on Windows)
       signtool sign /f cert.pfx /p "${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}" /fd SHA256 /tr http://timestamp.digicert.com target/x86_64-pc-windows-gnu/release/appgui.exe
       rm cert.pfx
   ```

### 4. macOS Code Signing & Notarization (Optional)

**Current Status**: Not configured - requires Apple Developer account
**To Enable macOS App Signing and Notarization**:

1. Enroll in Apple Developer Program:
   - Visit https://developer.apple.com/
   - Requires active account with signing capability
2. Create Code Signing Certificate:
   - Generate via Xcode or Apple Developer portal
   - Export as .p12 file with password
3. Create Provisioning Profile (if distributing outside App Store):
   - Create "Mac Distribution" provisioning profile
   - Export as .mobileprovision file
4. Store Credentials in GitHub Secrets:
   - `APPLE_DEVELOPER_ID`: Your Apple ID email
   - `APPLE_DEVELOPER_PASSWORD`: App-specific password
   - `APPLE_SIGNING_CERTIFICATE_P12_BASE64`: Base64-encoded .p12 file
   - `APPLE_SIGNING_CERTIFICATE_PASSWORD`: Certificate password
   - `APPLE_TEAM_ID`: Your Team ID (found in Developer portal)
5. Update macOS build step for code signing:
   ```yaml
   - name: Sign and notarize macOS app
     if: runner.os == 'macOS'
     run: |
       # Import certificate and sign
       codesign -s "Developer ID Application: Your Company" \
         ./target/aarch64-apple-darwin/release/appGUI
       # Notarize with Apple
       xcrun notarytool submit ./target/aarch64-apple-darwin/release/appGUI.dmg \
         --apple-id "${{ secrets.APPLE_DEVELOPER_ID }}" \
         --password "${{ secrets.APPLE_DEVELOPER_PASSWORD }}" \
         --team-id "${{ secrets.APPLE_TEAM_ID }}"
   ```

### 5. GitHub Release Permissions

**Current Status**: Configured (uses `GITHUB_TOKEN` with default permissions)
**Verification**:

- [ ] Repository is public (or workflow has write access to releases)
- [ ] GitHub Pages is enabled (Settings → Pages)
- [ ] Permissions allow: `contents: write` and `pages: write`
      **Current Permissions in Workflow**:

```yaml
permissions:
  contents: write # For creating releases
  pages: write # For Pages deployment
  id-token: write # For OIDC federation
```

### 6. Create Release Tag

**To Release v1.0.0**:

1. Update version if needed (already set to 1.0.0 in Cargo.toml):
   ```toml
   [workspace.package]
   version = "1.0.0"
   ```
2. Create and push tag to trigger workflow:
   ```bash
   git tag -a v1.0.0 -m "Release version 1.0.0"
   git push origin v1.0.0
   ```
3. Monitor workflow execution:
   - Go to Actions tab in GitHub
   - Watch the build-release workflow run
   - Check status of builds for all platforms
   - Release will be created when all jobs complete
4. Verify release was created:
   - Visit Releases page
   - Confirm all platform binaries are uploaded
   - Check SHA256SUMS file for integrity

### 7. Post-Release Tasks (Manual)

After the automated workflow completes:

1. **Test Downloads**:
   - [ ] Download each platform binary from the release
   - [ ] Verify checksums: `sha256sum -c SHA256SUMS`
   - [ ] Test basic functionality on representative systems
2. **Update Documentation**:
   - [ ] Update README with new version info
   - [ ] Add release notes to CHANGELOG.md
   - [ ] Update install instructions if URL format changed
3. **Distribution (if applicable)**:
   - [ ] Submit Android APK to Google Play Store (requires developer account)
   - [ ] Submit macOS app to App Store or distribute directly
   - [ ] Host Windows installer on distribution platform
4. **Announce Release**:
   - [ ] Create announcement post
   - [ ] Update website/landing pages
   - [ ] Notify users through appropriate channels

## Workflow Trigger Methods

### Method 1: Tag Push (Recommended for CI/CD)

```bash
# Create annotated tag
git tag -a v1.0.0 -m "Release v1.0.0: [description of changes]"
# Push tag to trigger workflow
git push origin v1.0.0
```

Workflow will:

- Checkout code at tag
- Build for all platforms
- Create GitHub release with all binaries
- Deploy to GitHub Pages

### Method 2: Repository Dispatch (For GitLab Integration)

```bash
curl -X POST https://api.github.com/repos/[owner]/[repo]/dispatches \
  -H "Authorization: token [GITHUB_TOKEN]" \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "build-release",
    "client_payload": {
      "gitlab_repo": "[gitlab-repo-url]",
      "tag": "v1.0.0"
    }
  }'
```

## Troubleshooting

### Android Build Fails

- Check `ANDROID_KEYSTORE_PASSWORD` secret is set
- Verify JDK 17 is available on runner
- Check Android SDK environment variables

### Windows Build Fails

- Verify MinGW toolchain is installed
- Check for PATH issues with cross-compilation tools

### macOS Build Fails

- Verify Xcode is installed (`xcode-select --install`)
- Check for provisioning profile issues
- Ensure Developer ID certificate is valid

### Release Not Created

- Check workflow logs in Actions tab
- Verify all build jobs completed successfully
- Ensure `GITHUB_TOKEN` has `contents: write` permission

## Security Considerations

1. **Keystore & Certificate Files**:
   - Never commit signing keys to repository
   - Use GitHub Secrets for sensitive credentials
   - Rotate keys periodically (annually minimum)
2. **GitHub Token**:
   - Use default `GITHUB_TOKEN` (automatically scoped)
   - Don't use personal access tokens unless necessary
   - Review token permissions regularly
3. **Credentials in Logs**:
   - Workflow uses masked secrets (won't print passwords)
   - Review action logs for any accidental exposures
   - Set private flag on repository if needed

## Timeline for v1.0.0 Release

- **Pre-Release**: Complete all manual setup above (1-2 days)
- **Build Phase**: Push tag, workflow runs (30-60 minutes per platform)
- **Testing Phase**: Download and verify binaries (2-4 hours)
- **Distribution**: Upload to app stores, update documentation (varies)
  **Estimated Total Time**: 8-24 hours (depending on testing and distribution)

## Checklist for Release

- [ ] Version confirmed as 1.0.0 in Cargo.toml
- [ ] All required GitHub Secrets configured
- [ ] Android keystore credentials stored securely
- [ ] Windows certificate set up (optional, but recommended)
- [ ] macOS signing configured (optional)
- [ ] Release notes prepared
- [ ] Tag created locally: `git tag -a v1.0.0 -m "Release v1.0.0"`
- [ ] Tag pushed to GitHub: `git push origin v1.0.0`
- [ ] Workflow runs successfully for all platforms
- [ ] All binaries downloaded and checksums verified
- [ ] Post-release tasks completed (testing, docs, distribution)

---

## Additional Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Cross-Compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Android Code Signing](https://developer.android.com/studio/publish/app-signing)
- [macOS Code Signing](https://developer.apple.com/support/code-signing/)
- [Windows Code Signing (Authenticode)](https://learn.microsoft.com/en-us/windows-hardware/drivers/dashboard/)
