# Quick Release Guide

This guide provides quick instructions for release process. For complete details, see [PLAN.md](./PLAN.md).

## Quick Start

### Step 1: Configure GitHub Secrets (Optional but Recommended)

For signing binaries (optional for first release), add to GitHub Repository Settings → Secrets and variables → Actions:

```
ANDROID_KEYSTORE_PASSWORD=<your-password>   # For APK signing
```

Optional (for production releases):

```
WINDOWS_CERTIFICATE_BASE64=<base64-encoded-pfx>
WINDOWS_CERTIFICATE_PASSWORD=<certificate-password>
WINDOWS_CERTIFICATE_COMPANY=<your-company>
```

See [PLAN.md](./PLAN.md) section "2. Android Release Keystore Setup" for details.

### Step 2: Create and Push Release Tag

```bash
# Create annotated tag
git tag -a v1.0.1 -m "Release version 1.0.1: Not production ready"
# Push to GitHub (triggers workflow)
git push origin v1.0.1
```

### Step 3: Monitor Build Progress

1. Go to your GitHub repository
2. Click **Actions** tab
3. Watch the "appGUI Build" workflow run
4. Wait for all platform builds to complete (~60 minutes)
5. Release page will be automatically created with all binaries

### Step 4: Verify Release

Once workflow completes:

1. Go to **Releases** page
2. Download binaries and checksums
3. Verify integrity:
   ```bash
   sha256sum -c SHA256SUMS
   ```

## What Gets Built

The workflow automatically builds:

| Platform       | Binary     | Location               |
| -------------- | ---------- | ---------------------- |
| Linux x86_64   | appGUI     | Linux executable       |
| Linux ARM64    | appGUI     | Linux ARM executable   |
| Windows x86_64 | appGUI.exe | Windows 10+ executable |
| macOS ARM64    | appGUI.app | Apple Silicon bundle   |
| Android        | appGUI.apk | Android APK package    |

## Workflow Features

- ✅ Multi-platform builds in parallel
- ✅ Automatic GitHub release creation
- ✅ SHA256 & MD5 checksums
- ✅ Deployment to GitHub Pages
- ✅ Release notes generation
- ✅ Optional code signing support

## Troubleshooting

**Build Failed?**

- Check Actions tab for error details
- See [PLAN.md](./PLAN.md) "Troubleshooting" section
  **Android Build Skipped?**
- Normal if `ANDROID_KEYSTORE_PASSWORD` secret not configured
- Set secret to enable APK signing
  **Release Not Created?**
- Ensure all build jobs completed successfully
- Check workflow permissions (Settings → Actions → General)

## After Release

1. **Download & Test**
   - Download binaries from release
   - Test on representative systems
   - Verify checksums
2. **Distribute** (Optional)
   - Android: Submit APK to Google Play Store
   - macOS: Submit to App Store or distribute directly
   - Linux: Host on download server
   - Windows: Create installer or distribute .exe
3. **Announce**
   - Update documentation/website
   - Notify users
   - Create changelog entry

## Next Steps for Future Releases

For subsequent releases (v1.0.2, v1.1.0, etc.):

1. Update version in `Cargo.toml` (if needed):
   ```toml
   [workspace.package]
   version = "1.0.2"
   ```
2. Update `CHANGELOG.md` with changes
3. Create and push tag:
   ```bash
   git tag -a v1.0.2 -m "Release version 1.0.2"
   git push origin v1.0.2
   ```

## Advanced Configuration

For advanced features like:

- Windows code signing (Authenticode)
- macOS notarization
- Custom build parameters
- GitLab integration
  See [PLAN.md](./PLAN.md) for detailed instructions.

---

**Questions?** See [PLAN.md](./PLAN.md) for comprehensive documentation.
