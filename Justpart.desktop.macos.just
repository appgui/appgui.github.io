#!/usr/bin/env -S just --justfile

# variables
_app_name_macos := "xPlatform"
_target_mac_arm64 := "aarch64-apple-darwin"

_is_supported_target_mac TARGET:
  #!/usr/bin/env sh
  if [ '{{TARGET}}' != '{{_target_mac_arm64}}' ]; then
    echo '[FAIL] "{{TARGET}}" is not a supported target'
    exit 1
  fi

[linux]
cc_to_mac_set_deps TARGET:
  #!/usr/bin/env sh
  just _is_supported_target_mac {{TARGET}} \
  || exit 1

  (
    just linux_debian_set_deps_cc_mac {{TARGET}} \
    || just linux_fedora_set_deps_cc_mac {{TARGET}}
  ) \ # install xcode somehow?
  && rustup target add {{TARGET}}

[windows]
cc_to_mac_set_deps TARGET:
  #!/usr/bin/env sh
  just _is_supported_target_mac {{TARGET}} \
  || exit 1

  just windows_set_deps_cc_mac {{TARGET}} \ # install xcode somehow?
  && rustup target add {{TARGET}}

[macos]
macos_set_deps:
  #!/usr/bin/env sh
  brew install imagemagick ninja
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
  && rustup toolchain install stable

[macos]
desktop_set_tools_dev:
  #!/usr/bin/env sh
  cargo install \
    slint-lsp \
      --no-default-features \
      --features backend-winit,renderer-skia
  brew install dprint

macos_build *ARGS:
  #!/usr/bin/env sh
  just macos build native {{ARGS}}

[macos]
macos_run *ARGS:
  #!/usr/bin/env sh
  just macos run native {{ARGS}}

macos ACTION='' TARGET='native' *ARGS:
  #!/usr/bin/env sh
  OUR_PLATFORM="$(just native)"
  if [ '{{TARGET}}' = 'native' ]; then
    export TARGET="${OUR_PLATFORM}"
  else
    echo "[INFO] Cross-compilation: ${OUR_PLATFORM} >> {{TARGET}}"
    just cc_to_mac_set_deps {{TARGET}} \
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

  command -v xcrun &>/dev/null || {
    echo '[FAIL] No Xcode found. Build will fail'
    exit 1
  }

  case '{{ACTION}}' in
    build)
      echo "[INFO] Attempting to build MacOS app bundle [${TARGET}]"
      # https://github.com/burtonageo/cargo-bundle
      cargo bundle -p feat_desktop --target ${TARGET} -r {{ARGS}} \
      && {
        echo '[DONE]'
        exit 0
      }

      echo "[INFO] Preparing MacOS executable [${TARGET}]"
      cargo build -p feat_desktop --target ${TARGET} -r {{ARGS}}
      ls -lh ./target/${TARGET}/release/{{_app_name_macos}}* | grep '^-rwxrwxr-x'
      echo '[DONE]'
      ;;
    *)
      if [ '{{ACTION}}' = 'run' ] \
      && [ "$(just native)" = "${TARGET}" ]; then
        cargo run {{ARGS}}
        exit 0
      fi

      if [ "$(just | grep 'macos_{{ACTION}}' | wc -l)" = '1' ]; then
        just macos_{{ACTION}} {{ARGS}}
      else
        echo '[INFO] Available actions:'
        just | grep macos_ | cut -c 11-
      fi
  esac
