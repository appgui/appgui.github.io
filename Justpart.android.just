#!/usr/bin/env -S just --justfile

# variables
DN_CN := env("DN_CN", 'Unknown')
DN_OU := env("DN_OU", 'Unknown')
DN_O := env("DN_O", 'Unknown')
DN_L := env("DN_L", 'Unknown')
DN_ST := env("DN_ST", 'Unknown')
DN_C := env("DN_C", 'XX')

is_amd64 := shell('if [ "$(uname -m)" = "x86_64" ]; then echo "1"; else echo "0"; fi')

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

# https://github.com/adoptium/temurin17-binaries/releases # at the moment Android works with JDK 17 only
openjdk_official_url_amd64 := "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.18+8/OpenJDK17U-jdk_x64_linux_hotspot_17.0.18_8.tar.gz"
openjdk_official_url_aarch64 := "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.18+8/OpenJDK17U-jdk_aarch64_linux_hotspot_17.0.18_8.tar.gz"

# Directory names
android_sdk_dir := ".android-sdk"
android_jdk_dir := ".android-jdk"
cmdline_tools_subdir := "cmdline-tools/latest"

# Cross-platform home directory: HOME on Unix, USERPROFILE on Windows
_home := if os() == "windows" { env('USERPROFILE') } else { env('HOME') }

# Paths
ANDROID_HOME := env('ANDROID_HOME', join(_home, android_sdk_dir))
JAVA_HOME := join(_home, android_jdk_dir, jdk_dir)
ANDROID_NDK_HOME := join(ANDROID_HOME, 'ndk', ndk_version)


setup_android_compiler:
  #!/bin/sh
  command -v rustup || sudo dnf install rustup
  rustup target add {{ android_target }} || rustup-init -t {{ android_target }} -y
  cargo install cargo-ndk
  echo '[DONE] Rust targets and cargo-ndk installed'

setup_android_sdk_compat: # allow running Android SDK (x86_64) on ARM64 host
  #!/bin/sh
  if [ '{{is_amd64}}' = '0' ]; then
    sudo sh -c '\
      dnf install qemu-user-static systemd-binfmt; \
      yes | dnf --installroot=/opt/x86_64-root --forcearch=x86_64 install \
        glibc zlib \
          --use-host-config; \
      \
    '
    export QEMU_LD_PREFIX=/opt/x86_64-root
    export LD_LIBRARY_PATH=/opt/x86_64-root/lib64

    mkdir -p .cargo 2> /dev/null || true
    export > .cargo/compat.env
  else
    echo '[INFO] Using native SDK binaries...'
  fi

setup_android_sdk:
  #!/bin/sh
  set -eu
  CMDLINE_TOOLS="${ANDROID_HOME}/{{ cmdline_tools_subdir }}"
  SDKMANAGER="${CMDLINE_TOOLS}/bin/sdkmanager"

  just setup_android_sdk_compat
  if [ -f .cargo/compat.env ]; then . .cargo/compat.env; fi

  $SDKMANAGER --version

  mkdir -p "${ANDROID_HOME}" || true

  if [ ! -f "${SDKMANAGER}" ]; then
    echo 'Installing Android SDK command-line tools...'
    mkdir -p "${CMDLINE_TOOLS}" || true
    cd /tmp
    wget -q '{{ android_sdk_cmdline_tools_url }}' -O cmdline-tools.zip
    unzip -q cmdline-tools.zip -d "${CMDLINE_TOOLS}"
    mv "${CMDLINE_TOOLS}/cmdline-tools"/* "${CMDLINE_TOOLS}/"
    rmdir "${CMDLINE_TOOLS}/cmdline-tools"
    rm cmdline-tools.zip
  fi

  export ANDROID_HOME ANDROID_SDK_ROOT="${ANDROID_HOME}"

  yes | "${SDKMANAGER}" --licenses || true
  "${SDKMANAGER}" 'platform-tools' 'build-tools;{{ build_tools_version }}' \
                  'platforms;{{ android_platform }}' 'ndk;{{ ndk_version }}'

  echo '[DONE] Android SDK/NDK installed'

setup_android_jdk:
  #!/bin/sh
  set -eu

  mkdir -p ~/{{ android_jdk_dir }} || true
  cd ~/{{ android_jdk_dir }}

  if [ -d '{{ jdk_dir }}' ]; then
    echo '[DONE] OpenJDK {{ java_version }} already installed'
    exit 0
  fi

  echo '[INFO] Downloading OpenJDK {{ java_version }}...'

  if [ '{{is_amd64}}' = '1' ]; then
    java_dl_link='{{ openjdk_official_url_amd64 }}'
  else
    java_dl_link='{{ openjdk_official_url_aarch64 }}'
  fi

  openjdk_archive_name=$(basename "$java_dl_link")
  wget -q "${java_dl_link}" -O "${openjdk_archive_name}"
  tar -xzf "${openjdk_archive_name}"

  openjdk_extracted_dir=$(ls | grep "{{ jdk_dir }}" | head -n1)
  mv "${openjdk_extracted_dir}" '{{ jdk_dir }}'
  rm "${openjdk_archive_name}"

  echo '[DONE] OpenJDK {{ java_version }} installed to ~/{{ android_jdk_dir }}/{{ jdk_dir }}'

setup_android_builder:
  #!/bin/sh
  cargo install cargo-apk2

setup_android_keystore:
  #!/bin/sh
  echo '[INFO] Setting up Android keystore...'

  if [ ! -d ./_android ]; then mkdir ./_android; fi
  cd ./_android

  set -e

  if [ -f ../.cargo/compat.env ]; then . ../.cargo/compat.env; fi || echo '[INFO] Not using compat mode'

  # Load secrets if exists
  if [ -f secrets.env ]; then
    . ./secrets.env
  fi

  # Set defaults or generate random values
  : "${KEY_ALIAS:=release}"
  : "${KEY_PASSWORD:=$(openssl rand -base64 32 2>/dev/null || echo 'changeit')}"

  rm ./release.keystore 2> /dev/null || true
  rm ./secrets.env 2> /dev/null || true

  echo """
  [package.metadata.android.signing.release]
  path = '"release.keystore"'
  keystore_password = '"${KEY_PASSWORD}"'
  key_alias = '"${KEY_ALIAS}"'
  key_password = '"${KEY_PASSWORD}"'

  """ > ../.cargo/keystore.toml

  # Save secrets if not already saved
  if [ ! -f ./secrets.env ]; then
    echo """# keystore values
  KEY_ALIAS="$KEY_ALIAS"
  KEY_PASSWORD="$KEY_PASSWORD"
  KEYSTORE_PASSWORD="$KEY_PASSWORD"
  DN_CN="{{DN_CN}}"
  DN_OU="{{DN_OU}}"
  DN_O="{{DN_O}}"
  DN_L="{{DN_L}}"
  DN_ST="{{DN_ST}}"
  DN_C="{{DN_C}}"
    """ > ./secrets.env
  fi

  # Generate keystore
  "${JAVA_HOME}/bin/keytool" -genkey -v \
    -keystore release.keystore \
    -alias "$KEY_ALIAS" \
    -keyalg RSA \
    -keysize 2048 \
    -validity 10000 \
    -storepass "$KEY_PASSWORD" \
    -keypass "$KEY_PASSWORD" \
    -dname "CN=$DN_CN, OU=$DN_OU, O=$DN_O, L=$DN_L, ST=$DN_ST, C=$DN_C"

setup_android_cargo_config:
  #!/bin/sh
  set -eu

  mkdir -p .cargo 2> /dev/null || true

  cat > .cargo/config.toml << 'EOF'
  [env]
  ANDROID_HOME = { value = "ANDROID_HOME_PLACEHOLDER", force = true }
  ANDROID_SDK_ROOT = { value = "ANDROID_HOME_PLACEHOLDER", force = true }
  ANDROID_NDK_HOME = { value = "NDK_HOME_PLACEHOLDER", force = true }
  ANDROID_JAR = { value = "ANDROID_HOME_PLACEHOLDER/platforms/ANDROID_PLATFORM_PLACEHOLDER/android.jar", force = true }
  ANDROID_PLATFORM = { value = "ANDROID_PLATFORM_VERSION_PLACEHOLDER", force = true }
  ANDROID_BUILD_TOOLS_VERSION = { value = "BUILD_TOOLS_VERSION_PLACEHOLDER", force = true }
  JAVA_HOME = { value = "JAVA_HOME_PLACEHOLDER", force = true }

  EOF

  sed -i "s|ANDROID_HOME_PLACEHOLDER|{{ ANDROID_HOME }}|g" .cargo/config.toml
  sed -i "s|NDK_HOME_PLACEHOLDER|{{ ANDROID_NDK_HOME }}|g" .cargo/config.toml
  sed -i "s|ANDROID_PLATFORM_PLACEHOLDER|{{ android_platform }}|g" .cargo/config.toml
  sed -i "s|ANDROID_PLATFORM_VERSION_PLACEHOLDER|{{ android_platform_version }}|g" .cargo/config.toml
  sed -i "s|BUILD_TOOLS_VERSION_PLACEHOLDER|{{ build_tools_version }}|g" .cargo/config.toml
  sed -i "s|JAVA_HOME_PLACEHOLDER|{{ JAVA_HOME }}|g" .cargo/config.toml

  echo '[DONE] Created .cargo/config.toml'

# Generate Android icon resources from assets/icon.png
setup_android_icons:
  #!/bin/sh
  set -e

  cd ./assets
  echo '[INFO] Generating Android icon resources...'

  # Check if source icon exists
  if [ ! -f ./icon.png ]; then
    just prepare_icons
  fi

  mkdir -p ./mipmap
  mkdir -p ./mipmap-mdpi
  mkdir -p ./mipmap-hdpi
  mkdir -p ./mipmap-xhdpi
  mkdir -p ./mipmap-xxhdpi
  mkdir -p ./mipmap-xxxhdpi

  # Check for image conversion tool
  if command -v convert >/dev/null 2>&1; then
    # Use ImageMagick
    echo '[INFO] Using ImageMagick to generate icons...'
    cp assets/icon.png ./mipmap/ic_launcher.png
    convert assets/icon.png -resize 48x48 ./mipmap-mdpi/ic_launcher.png
    convert assets/icon.png -resize 72x72 ./mipmap-hdpi/ic_launcher.png
    convert assets/icon.png -resize 96x96 ./mipmap-xhdpi/ic_launcher.png
    convert assets/icon.png -resize 144x144 ./mipmap-xxhdpi/ic_launcher.png
    convert assets/icon.png -resize 192x192 ./mipmap-xxxhdpi/ic_launcher.png
  elif command -v ffmpeg >/dev/null 2>&1; then
    # Use ffmpeg as fallback
    echo '[INFO] Using ffmpeg to generate icons...'
    ffmpeg -i assets/icon.png -vf scale=48:48 ./mipmap-mdpi/ic_launcher.png -y -loglevel error
    ffmpeg -i assets/icon.png -vf scale=72:72 ./mipmap-hdpi/ic_launcher.png -y -loglevel error
    ffmpeg -i assets/icon.png -vf scale=96:96 ./mipmap-xhdpi/ic_launcher.png -y -loglevel error
    ffmpeg -i assets/icon.png -vf scale=144:144 ./mipmap-xxhdpi/ic_launcher.png -y -loglevel error
    ffmpeg -i assets/icon.png -vf scale=192:192 ./mipmap-xxxhdpi/ic_launcher.png -y -loglevel error
  else
    echo '[ERROR] Neither ImageMagick (convert) nor ffmpeg found!'
    echo 'Install one of them:'
    echo '  Fedora/RHEL: sudo dnf install ImageMagick'
    echo '  Ubuntu/Debian: sudo apt install imagemagick'
    echo '  Arch: sudo pacman -S imagemagick'
    exit 1
  fi

  echo '[DONE] Android icons generated:'
  ls -lh ./mipmap-*/ic_launcher.png

setup_android:
  #!/bin/sh
  just setup_android_compiler
  just setup_android_sdk
  just setup_android_jdk
  just setup_android_builder
  just setup_android_keystore
  just setup_android_cargo_config
  just setup_android_icons
  echo '[DONE] Android setup finished'


# --- ANDROID ---

_exec_android ACTION *ARGS:
  #!/bin/sh
  cd ./feat_android
  set -e
  if [ -f ../.cargo/compat.env ]; then . ../.cargo/compat.env; else echo '[INFO] Not using compat mode'; fi
  export PATH="${JAVA_HOME}/bin:${PATH}"

  if [ ! -d "${ANDROID_NDK_HOME}" ] || [ ! -d "${JAVA_HOME}" ]; then
    echo '[INFO] Run setup first: just setup_all'
    exit 1
  fi

  if [ ! -f ../_android/release.keystore ]; then
    just setup_android_keystore
  fi

  # Generate icons if they don't exist
  if [ ! -f ../assets/mipmap-mdpi/ic_launcher.png ]; then
    echo '[INFO] Android icons not found, generating...'
    cd ..
    just setup_android_icons
    cd ./feat_android
  fi

  mkdir -p ../_android 2> /dev/null || true
  mkdir -p ../target 2> /dev/null || true

  if [ ! -f ../_android/src ] && [ ! -L ../_android/src ]; then
    ln -sf ../feat_android/src ../_android/src
  fi
  if [ ! -f ../_android/target ] && [ ! -L ../_android/target ]; then
    ln -sf ../target ../_android/target
  fi
  if [ ! -f ../_android/assets ] && [ ! -L ../_android/assets ]; then
    ln -sf ../assets ../_android/assets
  fi

  # Copy AndroidManifest.xml if it exists in feat_android
  # if [ -f ../feat_android/AndroidManifest.xml ]; then
  #   cp ../feat_android/AndroidManifest.xml ../_android/AndroidManifest.xml
  # fi

  echo '[package]' > ../_android/Cargo.toml
  echo 'name = "_android"' >> ../_android/Cargo.toml
  echo "$(tail -n +3 ../feat_android/Cargo.toml)" >> ../_android/Cargo.toml
  echo "$(cat ../.cargo/keystore.toml)" >> ../_android/Cargo.toml

  case '{{ACTION}}' in
    BUILD)
      echo '[INFO] Building Android APK (release, via cargo-apk2)...'
      cargo apk2 build --target {{ android_target }} --manifest-path ../_android/Cargo.toml --lib {{ARGS}}
      ;;
    RUN)
      echo '[INFO] Running Android APK (release, via cargo-apk2)...'
      cargo apk2 run --manifest-path ../_android/Cargo.toml --lib {{ARGS}}
      ;;
    *)
      echo '[FAIL] Unknown action'
  esac

# Run Android app via ADB
run_android *ARGS:
  #!/bin/sh
  just _exec_android RUN {{ARGS}}

# Build Android app installer
build_android *ARGS: # add --release to produce optimised archive
  #!/bin/sh
  just _exec_android BUILD {{ARGS}}
