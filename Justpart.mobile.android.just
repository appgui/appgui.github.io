#!/usr/bin/env -S just --justfile

# variables
_app_name_mobile := "xPlatform"

_DN_CN := env("DN_CN", 'Unknown')
_DN_OU := env("DN_OU", 'Unknown')
_DN_O := env("DN_O", 'Unknown')
_DN_L := env("DN_L", 'Unknown')
_DN_ST := env("DN_ST", 'Unknown')
_DN_C := env("DN_C", 'XX')

_is_amd64 := shell('if [ "$(uname -m)" = "x86_64" ]; then echo "1"; else echo "0"; fi')

# Build targets and versions
_android_target := "aarch64-linux-android"
_ndk_version := "27.2.12479018"
_android_platform := "android-34"
_android_platform_version := "34"
_android_build_tools_version := "34.0.0"
_java_version := "17"
_jdk_dir := "jdk-17"

# Official URLs
_android_sdk_cmdline_tools_url := "https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip"

# https://github.com/adoptium/temurin17-binaries/releases # at the moment Android works with JDK 17 only
_openjdk_official_url_amd64 := "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.18+8/OpenJDK17U-jdk_x64_linux_hotspot_17.0.18_8.tar.gz"
_openjdk_official_url_aarch64 := "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.18+8/OpenJDK17U-jdk_aarch64_linux_hotspot_17.0.18_8.tar.gz"

# Directory names
_android_sdk_dir := ".android-sdk"
_android_jdk_dir := ".android-jdk"
_android_cmdline_tools_subdir := "cmdline-tools/latest"

# Cross-platform home directory: HOME on Unix, USERPROFILE on Windows
_home := if os() == "windows" { env('USERPROFILE') } else { env('HOME') }

[private]
ANDROID_HOME := env('ANDROID_HOME', join(_home, _android_sdk_dir))

[private]
JAVA_HOME := join(_home, _android_jdk_dir, _jdk_dir)

[private]
ANDROID_NDK_HOME := join(ANDROID_HOME, 'ndk', _ndk_version)


android_setup_compiler:
  #!/usr/bin/env sh
  command -v rustup || sudo dnf install rustup
  rustup target add {{ _android_target }} || rustup-init -t {{ _android_target }} -y
  cargo install cargo-apk2 cargo-ndk
  echo '[DONE] Rust targets and cargo-ndk installed'

android_setup_sdk_compat: # allow running Android SDK (x86_64) on ARM64 host
  #!/usr/bin/env sh
  if [ '{{_is_amd64}}' = '0' ]; then
    sudo sh -c '\
      dnf install qemu-user-static systemd-binfmt; \
      yes | dnf --installroot=/opt/x86_64-root --forcearch=x86_64 install \
        glibc zlib \
          --use-host-config; \
      \
    '
    export QEMU_LD_PREFIX=/opt/x86_64-root
    export LD_LIBRARY_PATH=/opt/x86_64-root/lib64

    mkdir -p ./.cargo 2> /dev/null || true
    export > ./.env
  else
    echo '[INFO] Using native SDK binaries...'
  fi

android_setup_sdk:
  #!/usr/bin/env sh
  set -eu
  CMDLINE_TOOLS="${ANDROID_HOME}/{{ _android_cmdline_tools_subdir }}"
  SDKMANAGER="${CMDLINE_TOOLS}/bin/sdkmanager"

  just android_setup_sdk_compat

  if [ -f ./.env ]; then
    . ./.env
  fi

  mkdir -p "${ANDROID_HOME}" || true

  if [ ! -f "${SDKMANAGER}" ]; then
    echo 'Installing Android SDK command-line tools...'
    mkdir -p "${CMDLINE_TOOLS}" || true
    cd /tmp
    wget -q '{{ _android_sdk_cmdline_tools_url }}' -O cmdline-tools.zip
    unzip -q cmdline-tools.zip -d "${CMDLINE_TOOLS}"
    mv "${CMDLINE_TOOLS}/cmdline-tools"/* "${CMDLINE_TOOLS}/"
    rmdir "${CMDLINE_TOOLS}/cmdline-tools"
    rm cmdline-tools.zip
  fi

  echo "[INFO] sdkmanager version: $($SDKMANAGER --version)"

  export ANDROID_HOME ANDROID_SDK_ROOT="${ANDROID_HOME}"

  yes | "${SDKMANAGER}" --licenses || true
  "${SDKMANAGER}" 'platform-tools' 'build-tools;{{ _android_build_tools_version }}' \
                  'platforms;{{ _android_platform }}' 'ndk;{{ _ndk_version }}'

  echo '[DONE] Android SDK/NDK installed'

android_setup_jdk:
  #!/usr/bin/env sh
  set -eu

  mkdir -p ~/{{ _android_jdk_dir }} || true
  cd ~/{{ _android_jdk_dir }}

  if [ -d '{{ _jdk_dir }}' ]; then
    echo '[DONE] OpenJDK {{ _java_version }} already installed'
    exit 0
  fi

  echo '[INFO] Downloading OpenJDK {{ _java_version }}...'

  if [ '{{_is_amd64}}' = '1' ]; then
    java_dl_link='{{ _openjdk_official_url_amd64 }}'
  else
    java_dl_link='{{ _openjdk_official_url_aarch64 }}'
  fi

  openjdk_archive_name=$(basename "$java_dl_link")
  wget -q "${java_dl_link}" -O "${openjdk_archive_name}"
  tar -xzf "${openjdk_archive_name}"

  openjdk_extracted_dir=$(ls | grep "{{ _jdk_dir }}" | head -n1)
  mv "${openjdk_extracted_dir}" '{{ _jdk_dir }}'
  rm "${openjdk_archive_name}"

  echo '[DONE] OpenJDK {{ _java_version }} installed to ~/{{ _android_jdk_dir }}/{{ _jdk_dir }}'

android_setup_keystore:
  #!/usr/bin/env sh
  echo '[INFO] Setting up Android keystore...'

  cd ./feat_mobile
  set -e
  if [ -f ../.env ]; then
    . ../.env
  fi

  # Load env if exists
  if [ -f ./secret.env ]; then
    . ./secret.env
  fi

  # Set defaults or generate random values
  : "${KEY_ALIAS:=release}"
  : "${KEY_PASSWORD:=$(openssl rand -base64 32 2>/dev/null || echo 'changeit')}"

  rm ./release.keystore 2> /dev/null || true
  rm ./secret.env 2> /dev/null || true

  echo """
  [package.metadata.android.signing.release]
  path = '"release.keystore"'
  keystore_password = '"${KEY_PASSWORD}"'
  key_alias = '"${KEY_ALIAS}"'
  key_password = '"${KEY_PASSWORD}"'
  """ > ../.cargo/keystore.toml

  # Save env if not already saved
  if [ ! -f ./secret.env ]; then
    echo """# keystore values
  KEY_ALIAS="$KEY_ALIAS"
  KEY_PASSWORD="$KEY_PASSWORD"
  KEYSTORE_PASSWORD="$KEY_PASSWORD"
  DN_CN="{{_DN_CN}}"
  DN_OU="{{_DN_OU}}"
  DN_O="{{_DN_O}}"
  DN_L="{{_DN_L}}"
  DN_ST="{{_DN_ST}}"
  DN_C="{{_DN_C}}"
    """ > ./secret.env
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

android_setup_cargo_config:
  #!/usr/bin/env sh
  set -eu

  mkdir -p ./.cargo 2> /dev/null || true

  cat > ./.cargo/config.toml << 'EOF'

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
  sed -i "s|ANDROID_PLATFORM_PLACEHOLDER|{{ _android_platform }}|g" .cargo/config.toml
  sed -i "s|ANDROID_PLATFORM_VERSION_PLACEHOLDER|{{ _android_platform_version }}|g" .cargo/config.toml
  sed -i "s|BUILD_TOOLS_VERSION_PLACEHOLDER|{{ _android_build_tools_version }}|g" .cargo/config.toml
  sed -i "s|JAVA_HOME_PLACEHOLDER|{{ JAVA_HOME }}|g" .cargo/config.toml

  echo '[DONE] Created .cargo/config.toml'

# Generate Android icon resources from assets/icon.svg
android_icons_make:
  #!/usr/bin/env sh
  set -e

  cd ./assets
  echo '[INFO] Generating Android icon resources...'

  # Check if source icon exists
  if [ ! -f ./icon.png ]; then
    magick -background transparent -density 1200 ./icon.svg \
      -resize 512x512 \
        ./icon.png
  fi

  mkdir -p ./mipmap
  mkdir -p ./mipmap-mdpi
  mkdir -p ./mipmap-hdpi
  mkdir -p ./mipmap-xhdpi
  mkdir -p ./mipmap-xxhdpi
  mkdir -p ./mipmap-xxxhdpi

  cp ./icon.png ./mipmap/ic_launcher.png
  magick ./icon.png -resize 48x48 ./mipmap-mdpi/ic_launcher.png
  magick ./icon.png -resize 72x72 ./mipmap-hdpi/ic_launcher.png
  magick ./icon.png -resize 96x96 ./mipmap-xhdpi/ic_launcher.png
  magick ./icon.png -resize 144x144 ./mipmap-xxhdpi/ic_launcher.png
  magick ./icon.png -resize 192x192 ./mipmap-xxxhdpi/ic_launcher.png

  echo '[DONE] Android icons generated:'
  ls -lh ./mipmap-*/ic_launcher.png


android_setup:
  #!/usr/bin/env sh
  just android_setup_compiler
  just android_setup_sdk
  just android_setup_jdk
  just android_setup_keystore
  just android_setup_cargo_config

  just android_icons_make

  echo '[DONE] Android setup finished'

android_build *ARGS:
  #!/usr/bin/env sh
  android build {{ARGS}}

android ACTION='' *ARGS:
  #!/usr/bin/env sh
  cd ./feat_mobile
  set -e
  if [ -f ../.env ]; then
    . ../.env
  fi
  export PATH="${JAVA_HOME}/bin:${PATH}"

  if [ ! -d "${ANDROID_NDK_HOME}" ] || [ ! -d "${JAVA_HOME}" ]; then
    echo '[INFO] Run setup first: just android_setup'
    exit 1
  fi

  if [ ! -f ./release.keystore ]; then
    just android_setup_keystore
  fi

  # Generate icons if they don't exist
  if [ ! -f ../assets/mipmap-mdpi/ic_launcher.png ]; then
    echo '[INFO] Android icons not found, generating...'
    cd ..
    just android_icons_make
    cd ./feat_mobile
  fi

  # Copy AndroidManifest.xml if it exists in feat_mobile
  # if [ -f ../feat_mobile/AndroidManifest.xml ]; then
  #   cp ../feat_mobile/AndroidManifest.xml ../feat_mobile/AndroidManifest.xml
  # fi

  echo "$(cat ./Cargo.base.toml)" > ./Cargo.toml
  echo "$(cat ../.cargo/keystore.toml)" >> ./Cargo.toml

  case '{{ACTION}}' in
    build)
      echo '[INFO] Preparing Android package'
      cargo apk2 build --target {{ _android_target }} -r --lib {{ARGS}}
      ls -lh 'target/release/apk/{{_app_name_mobile}}.apk'
      ;;
    run)
      echo '[INFO] Running Android APK (release, via cargo-apk2)...'
      cargo apk2 run --lib {{ARGS}}
      ;;
    *)
      if [ "$(just | grep 'android_{{ACTION}}' | wc -l)" = '1' ]; then
        just android_{{ACTION}} {{ARGS}}
      else
        echo '[INFO] Available actions:'
        just | grep android_ | cut -c 13-
        echo 'run                             # natively on an attached device'
      fi
  esac
  # git update-index --skip-worktree ../feat_mobile/Cargo.release.toml
