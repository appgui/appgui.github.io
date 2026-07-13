#!/usr/bin/env -S just --justfile

# variables
_app_name_linux := "xPlatform"
_target_lin_amd64 := "x86_64-unknown-linux-gnu"
_target_lin_arm64 := "aarch64-unknown-linux-gnu"

_is_supported_target_lin TARGET:
  #!/usr/bin/env sh
  if [ '{{TARGET}}' != '{{_target_lin_amd64}}' ] \
  || [ '{{TARGET}}' != '{{_target_lin_arm64}}' ]; then
    echo '[FAIL] "{{TARGET}}" is not a supported target'
    exit 1
  fi

[macos]
cc_to_lin_set_deps TARGET:
  #!/usr/bin/env sh
  just _is_supported_target_lin {{TARGET}} \
  || exit 1

  just macos_set_deps_cc_lin {{TARGET}} \
  && rustup target add {{TARGET}}

[windows]
cc_to_lin_set_deps TARGET:
  #!/usr/bin/env sh
  just _is_supported_target_lin {{TARGET}} \
  || exit 1

  just windows_set_deps_cc_lin {{TARGET}} \
  && rustup target add {{TARGET}}

[linux]
linux_debian_set_deps:
  #!/usr/bin/env sh
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y \
    --no-install-recommends --fix-missing \
      libfontconfig1-dev libssl-dev libdbus-1-dev libgtk-3-dev pkg-config rustup
  cargo install \
    --git https://github.com/burtonageo/cargo-bundle \
    --branch master \
      cargo-bundle

[linux]
desktop_set_tools_dev:
  #!/usr/bin/env sh
  cargo install \
    slint-lsp \
      --no-default-features \
      --features backend-winit,renderer-femtovg,renderer-skia
  curl -fsSL https://dprint.dev/install.sh | sh

[linux]
linux_debian_set_deps_cc_mac:
  #!/usr/bin/env sh
  command -v apt
  echo 'TODO: somehow get xcode alternative'

[linux]
linux_debian_set_deps_cc_win:
  #!/usr/bin/env sh
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y \
    --no-install-recommends --fix-missing \
      gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64 binutils-mingw-w64-x86-64

[linux]
linux_fedora_set_deps:
  #!/usr/bin/env sh
  sudo su -c 'yes | dnf install \
    fontconfig-devel openssl-devel pkg-config rustup'
  cargo install \
    --git https://github.com/burtonageo/cargo-bundle \
    --branch master \
      cargo-bundle

[linux]
linux_opensuse_set_deps:
  #!/usr/bin/env sh
  sudo su -c 'zypper install -y \
    fontconfig-devel openssl-devel pkg-config rustup squashfs'
  cargo install \
    --git https://github.com/burtonageo/cargo-bundle \
    --branch master \
      cargo-bundle

[linux]
linux_fedora_set_deps_cc_mac:
  #!/usr/bin/env sh
  command -v dnf
  echo 'TODO: somehow get xcode alternative'

[linux]
linux_fedora_set_deps_cc_win:
  #!/usr/bin/env sh
  sudo su -c 'yes | dnf install mingw64-gcc mingw64-gcc-c++ mingw64-binutils'
  echo 'TODO'

linux_build TARGET *ARGS:
  #!/usr/bin/env sh
  just linux build {{TARGET}} {{ARGS}}

[linux]
linux_run *ARGS:
  #!/usr/bin/env sh
  just linux run native {{ARGS}}

linux ACTION='' TARGET='native' *ARGS:
  #!/usr/bin/env sh
  OUR_PLATFORM="$(just native)"
  if [ '{{TARGET}}' = 'native' ]; then
    export TARGET="${OUR_PLATFORM}"
  else
    echo "[INFO] Cross-compilation: ${OUR_PLATFORM} >> {{TARGET}}"
    just cc_to_lin_set_deps {{TARGET}} \
    || exit 1
    export TARGET={{TARGET}}
  fi

  echo "[INFO] Target: ${TARGET}"

  just _workspace_set
  if [ ! -f ./assets/icon.ico ]; then
    just desktop_icons_make
  fi

  cd ./feat_desktop
  set -e
  if [ -f ../.env ]; then
    . ../.env
  fi

  case '{{ACTION}}' in
    build)
      echo "[INFO] Attempting to build Linux installers and appimage [${TARGET}]"
      # https://github.com/burtonageo/cargo-bundle
      cargo bundle -p feat_desktop --target ${TARGET} -r {{ARGS}} \
      && {
        echo '[DONE]'
        exit 0
      }

      echo "[INFO] Preparing Linux executable [${TARGET}]"
      cargo build -p feat_desktop --target ${TARGET} -r {{ARGS}}
      ls -lh ./target/${TARGET}/release/{{_app_name_linux}}* | grep '^-rwxrwxr-x'
      echo '[DONE]'
      ;;
    *)
      if [ '{{ACTION}}' = 'run' ] \
      && [ "${OUR_PLATFORM}" = "${TARGET}" ]; then
        cargo run {{ARGS}}
        exit 0
      fi

      if [ "$(just | grep 'linux_{{ACTION}}' | wc -l)" = '1' ]; then
        just linux_{{ACTION}} {{ARGS}}
      else
        echo '[INFO] Available actions:'
        just | grep linux_ | cut -c 11-
      fi
  esac
