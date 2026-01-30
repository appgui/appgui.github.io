# SlintXPlatform - Project Command Runner

set dotenv-load
set export
set quiet := true

_list:
  just --list

# Variables
app_name := "slintxplatform"

# Build targets and versions
android_target := "aarch64-linux-android"
ndk_version := "27.2.12479018"
android_platform := "android-34"
android_platform_version := "34"
build_tools_version := "34.0.0"
java_version := "17"

# Official URLs
android_sdk_cmdline_tools_url := "https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip"
openjdk_official_url := "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_linux-x64_bin.tar.gz"
openjdk_archive_name := "openjdk-17.0.2_linux-x64_bin.tar.gz"
openjdk_extracted_dir := "jdk-17.0.2"

# Directory names
android_sdk_dir := ".android-sdk"
android_jdk_dir := ".android-jdk"
jdk_install_name := "jdk-17"
cmdline_tools_subdir := "cmdline-tools/latest"

# Paths
ANDROID_HOME := env('ANDROID_HOME', join(env('HOME'), android_sdk_dir))
JAVA_HOME := join(env('HOME'), android_jdk_dir, jdk_install_name)
ANDROID_NDK_HOME := join(ANDROID_HOME, "ndk", ndk_version)

# --- SETUP ---

setup:
  #!/bin/sh
  rustup target add {{ android_target }}
  cargo install cargo-ndk
  echo "✓ Rust targets and cargo-ndk installed"

setup-android-sdk:
  #!/bin/sh
  set -euo pipefail
  CMDLINE_TOOLS="${ANDROID_HOME}/{{ cmdline_tools_subdir }}"
  SDKMANAGER="${CMDLINE_TOOLS}/bin/sdkmanager"

  mkdir -p "${ANDROID_HOME}"

  if [ ! -f "${SDKMANAGER}" ]; then
    echo "Installing Android SDK command-line tools..."
    mkdir -p "${CMDLINE_TOOLS}"
    cd /tmp
    wget -q "{{ android_sdk_cmdline_tools_url }}" -O cmdline-tools.zip
    unzip -q cmdline-tools.zip -d "${CMDLINE_TOOLS}"
    mv "${CMDLINE_TOOLS}/cmdline-tools"/* "${CMDLINE_TOOLS}/"
    rmdir "${CMDLINE_TOOLS}/cmdline-tools"
    rm cmdline-tools.zip
  fi

  export ANDROID_HOME ANDROID_SDK_ROOT="${ANDROID_HOME}"

  yes | "${SDKMANAGER}" --licenses || true
  "${SDKMANAGER}" "platform-tools" "build-tools;{{ build_tools_version }}" \
                  "platforms;{{ android_platform }}" "ndk;{{ ndk_version }}"

  echo "✓ Android SDK/NDK installed"

setup-android-jdk:
  #!/bin/sh
  set -euo pipefail

  mkdir -p ~/{{ android_jdk_dir }}
  cd ~/{{ android_jdk_dir }}

  if [ -d "{{ jdk_install_name }}" ]; then
    echo "✓ OpenJDK {{ java_version }} already installed"
    exit 0
  fi

  echo "Downloading OpenJDK {{ java_version }}..."
  wget -q "{{ openjdk_official_url }}" -O "{{ openjdk_archive_name }}"
  tar -xzf "{{ openjdk_archive_name }}"
  mv {{ openjdk_extracted_dir }} {{ jdk_install_name }}
  rm "{{ openjdk_archive_name }}"

  echo "✓ OpenJDK {{ java_version }} installed to ~/{{ android_jdk_dir }}/{{ jdk_install_name }}"

setup-android-builder:
  #!/bin/sh
  cargo install cargo-apk2

setup-cargo-config:
  #!/bin/sh
  set -euo pipefail

  mkdir -p .cargo

  cat > .cargo/config.toml << 'EOF'
  [env]
  ANDROID_HOME = { value = "ANDROID_HOME_PLACEHOLDER", force = true }
  ANDROID_SDK_ROOT = { value = "ANDROID_HOME_PLACEHOLDER", force = true }
  ANDROID_NDK_HOME = { value = "NDK_HOME_PLACEHOLDER", force = true }
  ANDROID_JAR = { value = "ANDROID_HOME_PLACEHOLDER/platforms/ANDROID_PLATFORM_PLACEHOLDER/android.jar", force = true }
  ANDROID_PLATFORM = { value = "ANDROID_PLATFORM_VERSION_PLACEHOLDER", force = true }
  ANDROID_BUILD_TOOLS_VERSION = { value = "BUILD_TOOLS_VERSION_PLACEHOLDER", force = true }
  JAVA_HOME = { value = "JAVA_HOME_PLACEHOLDER", force = true }

  [package.metadata.android.signing.release] # TODO
  path = "release.keystore"
  keystore_password = "android"
  key_alias = "release"
  key_password = "android"
  EOF

  sed -i "s|ANDROID_HOME_PLACEHOLDER|{{ ANDROID_HOME }}|g" .cargo/config.toml
  sed -i "s|NDK_HOME_PLACEHOLDER|{{ ANDROID_NDK_HOME }}|g" .cargo/config.toml
  sed -i "s|ANDROID_PLATFORM_PLACEHOLDER|{{ android_platform }}|g" .cargo/config.toml
  sed -i "s|ANDROID_PLATFORM_VERSION_PLACEHOLDER|{{ android_platform_version }}|g" .cargo/config.toml
  sed -i "s|BUILD_TOOLS_VERSION_PLACEHOLDER|{{ build_tools_version }}|g" .cargo/config.toml
  sed -i "s|JAVA_HOME_PLACEHOLDER|{{ JAVA_HOME }}|g" .cargo/config.toml

  echo "✓ Created .cargo/config.toml"

setup-all:
  #!/bin/sh
  just setup
  just setup-android-sdk
  just setup-android-jdk
  just setup-cargo-config
  just setup-android-builder
  echo "✓ Complete setup finished"

# --- DESKTOP ---

run:
  #!/bin/sh
  cargo run -p feat_desktop

build-desktop:
  #!/bin/sh
  cargo build -p feat_desktop --release

# --- ANDROID ---

build-android-libs FLAVOUR="--release": # or empty
  #!/bin/sh
  set -e
  export PATH="${JAVA_HOME}/bin:${PATH}"

  if [ ! -d "${ANDROID_NDK_HOME}" ]; then
    echo "❌ Android NDK not found. Run: just setup-android-sdk"
    exit 1
  fi

  if [ ! -d "${JAVA_HOME}" ]; then
    echo "❌ Android JDK not found. Run: just setup-android-jdk"
    exit 1
  fi

  echo "Building for {{ android_target }}..."
  cargo ndk -t {{ android_target }} build --lib {{FLAVOUR}}

# Run Android app via ADB
run-android *ARGS:
  #!/bin/sh
  cd ./feat_android
  set -e
  export PATH="${JAVA_HOME}/bin:${PATH}"

  if [ ! -d "${ANDROID_NDK_HOME}" ] || [ ! -d "${JAVA_HOME}" ]; then
    echo "❌ Run setup first: just setup-all"
    exit 1
  fi
  echo "Building installable Android APK (release, via cargo-apk2)..."
  cargo apk2 run {{ARGS}}

# Build Android app installer
build-android-apk *ARGS:
  #!/bin/sh
  cd ./feat_android
  set -e
  export PATH="${JAVA_HOME}/bin:${PATH}"

  if [ ! -d "${ANDROID_NDK_HOME}" ] || [ ! -d "${JAVA_HOME}" ]; then
    echo "❌ Run setup first: just setup-all"
    exit 1
  fi
  echo "Building installable Android APK (release, via cargo-apk2)..."
  cargo apk2 build --target {{ android_target }} {{ARGS}}

# --- UTILS ---

check:
  #!/bin/sh
  cargo fmt --all
  cargo clippy --all-targets -- -D warnings

clean:
  #!/bin/sh
  cargo clean

clean-all:
  #!/bin/sh
  cargo clean
  rm -rf {{ ANDROID_HOME }} ~/{{ android_jdk_dir }}
  echo "✓ Cleaned everything"

release:
  #!/bin/sh
  echo 'Desktop'
  just build-desktop
  bin_desktop_app='target/release/{{app_name}}'
  ls -lh ${bin_desktop_app}* | grep "^-rwxrwxr-x"
  echo

  echo 'Mobile'
  just build-android-apk --release
