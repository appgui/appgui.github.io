# SlintXPlatform - Project Command Runner

set dotenv-load
set export
set quiet := true

_list:
  just --list

# Variables
app_name := "slintxplatform"

is_amd64 := shell('if [ "$(uname -m)" == "x86_64" ]; then echo "1"; else echo "0"; fi')

# Build targets and versions
android_target := "aarch64-linux-android"
ndk_version := "27.2.12479018"
android_platform := "android-34"
android_platform_version := "34"
build_tools_version := "34.0.0"
java_version := "17"
jdk_dir := "jdk-17"

# Official URLs
android_sdk_cmdline_tools_url := "https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip"

openjdk_official_url_amd64 := "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.18+8/OpenJDK17U-jdk_x64_linux_hotspot_17.0.18_8.tar.gz"
openjdk_official_url_aarch64 := "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.18+8/OpenJDK17U-jdk_aarch64_linux_hotspot_17.0.18_8.tar.gz"

# Directory names
android_sdk_dir := ".android-sdk"
android_jdk_dir := ".android-jdk"
cmdline_tools_subdir := "cmdline-tools/latest"

# Paths
ANDROID_HOME := env('ANDROID_HOME', join(env('HOME'), android_sdk_dir))
JAVA_HOME := join(env('HOME'), android_jdk_dir, jdk_dir)
ANDROID_NDK_HOME := join(ANDROID_HOME, "ndk", ndk_version)

env:
  #!/bin/sh
  env

# --- SETUP ---

setup:
  #!/bin/sh
  command -v rustup || sudo dnf install rustup
  rustup target add {{ android_target }} || rustup-init -t {{ android_target }} -y
  cargo install cargo-ndk
  echo "[DONE] Rust targets and cargo-ndk installed"
  
setup-compat:
  #!/bin/sh
  if [ "{{is_amd64}}" -eq "0" ]; then
    sudo sh -c "\
      dnf install qemu-user-static systemd-binfmt; \
      yes | dnf --installroot=/opt/x86_64-root --forcearch=x86_64 install \
        glibc zlib \
          --use-host-config; \
      \
    "
    export QEMU_LD_PREFIX=/opt/x86_64-root
    export LD_LIBRARY_PATH=/opt/x86_64-root/lib64
    
    mkdir -p .cargo 2> /dev/null
    export > .cargo/compat.env
  else
    echo '[INFO] Using native SDK binaries...'
  fi

setup-android-sdk:
  #!/bin/sh
  set -euo pipefail
  CMDLINE_TOOLS="${ANDROID_HOME}/{{ cmdline_tools_subdir }}"
  SDKMANAGER="${CMDLINE_TOOLS}/bin/sdkmanager"
  
  just setup-compat
  eval $(cat .cargo/compat.env)

  $SDKMANAGER --version

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

  echo "[DONE] Android SDK/NDK installed"

setup-android-jdk:
  #!/bin/sh
  set -euo pipefail

  mkdir -p ~/{{ android_jdk_dir }}
  cd ~/{{ android_jdk_dir }}

  if [ -d "{{ jdk_dir }}" ]; then
    echo "[DONE] OpenJDK {{ java_version }} already installed"
    exit 0
  fi

  echo "Downloading OpenJDK {{ java_version }}..."
  
  if [ "{{is_amd64}}" -eq "1" ]; then
    java_dl_link="{{ openjdk_official_url_amd64 }}"
  else
    java_dl_link="{{ openjdk_official_url_aarch64 }}"
  fi
  
  openjdk_archive_name=$(basename $java_dl_link)
  wget -q "${java_dl_link}" -O "${openjdk_archive_name}"
  tar -xzf "${openjdk_archive_name}"
  
  openjdk_extracted_dir=$(ls | grep {{ jdk_dir }} | head -n1)
  mv "${openjdk_extracted_dir}" "{{ jdk_dir }}"
  rm "${openjdk_archive_name}"

  echo "[DONE] OpenJDK {{ java_version }} installed to ~/{{ android_jdk_dir }}/{{ jdk_dir }}"

setup-android-builder:
  #!/bin/sh
  cargo install cargo-apk2

setup-cargo-config:
  #!/bin/sh
  set -euo pipefail

  mkdir -p .cargo 2> /dev/null

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

  echo "[DONE] Created .cargo/config.toml"

setup-all:
  #!/bin/sh
  just setup
  just setup-android-sdk
  just setup-android-jdk
  just setup-cargo-config
  just setup-android-builder
  echo "[DONE] Complete setup finished"

# --- DESKTOP ---

run:
  #!/bin/sh
  cargo run -p feat_desktop

build-desktop *ARGS: # add --release to produce optimised binary
  #!/bin/sh
  cargo build -p feat_desktop {{ARGS}}

# --- ANDROID ---

# Run Android app via ADB
run-android *ARGS:
  #!/bin/sh
  cd ./feat_android
  set -e
  eval $(cat ../.cargo/compat.env) || echo '[INFO] Not using compat mode'
  export PATH="${JAVA_HOME}/bin:${PATH}"

  if [ ! -d "${ANDROID_NDK_HOME}" ] || [ ! -d "${JAVA_HOME}" ]; then
    echo "[INFO] Run setup first: just setup-all"
    exit 1
  fi
  echo "[INFO] Building installable Android APK (release, via cargo-apk2)..."
  cargo apk2 run {{ARGS}}

# Build Android app installer
build-android-apk *ARGS: # add --release to produce optimised archive
  #!/bin/sh
  cd ./feat_android
  set -e
  eval $(cat ../.cargo/compat.env) || echo '[INFO] Not using compat mode'
  export PATH="${JAVA_HOME}/bin:${PATH}"

  if [ ! -d "${ANDROID_NDK_HOME}" ] || [ ! -d "${JAVA_HOME}" ]; then
    echo "[INFO] Run setup first: just setup-all"
    exit 1
  fi
  echo "Building installable Android APK (release, via cargo-apk2)..."
  cargo apk2 build --target {{ android_target }} {{ARGS}}

# --- UTILS ---

clean:
  #!/bin/sh
  cargo clean

release:
  #!/bin/sh
  echo 'Desktop'
  just build-desktop --release
  ls -lh ./target/release/{{app_name}}* | grep "^-rwxrwxr-x"
  echo

  echo 'Mobile'
  just build-android-apk --release
  ls -lh 'target/release/apk/feat_android.apk'
