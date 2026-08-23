#!/usr/bin/env -S just --justfile

_app_name_windows := "xplatform"
_target_win_amd64 := 'x86_64-pc-windows-gnu'
_target_win_amd64_proprietary := 'x86_64-pc-windows-msvc'
_target_win_arm64 := 'aarch64-pc-windows-msvc' # Windows on ARM, not tested

_is_supported_target_win TARGET:
  #!/usr/bin/env sh
  if [ '{{TARGET}}' != '{{_target_win_amd64}}' ] \
  || [ '{{TARGET}}' != '{{_target_win_amd64_proprietary}}' ] || \
     [ '{{TARGET}}' != '{{_target_win_arm64}}' ]; then
    echo '[FAIL] "{{TARGET}}" is not a supported target'
    exit 1
  fi

[linux]
cc_to_win_set_deps TARGET:
  #!/usr/bin/env sh
  just _is_supported_target_win {{TARGET}} \
  || exit 1

  if [ '{{TARGET}}' = '{{_target_win_arm64}}' ]; then
    cargo install cargo-xwin
  fi
  (
    just linux_debian_set_deps_cc_win \
    || just linux_fedora_set_deps_cc_win
  ) \
  && rustup target add {{TARGET}}

[macos]
cc_to_win_set_deps TARGET:
  #!/usr/bin/env sh
  just _is_supported_target_win {{TARGET}} \
  || exit 1

  if [ '{{TARGET}}' = '{{_target_win_arm64}}' ]; then
    cargo install cargo-xwin
  fi

  just macos_set_deps_cc_win \
  && rustup target add {{TARGET}}

[windows]
windows_set_deps:
  #!pwsh
  winget install -e --id Rustlang.Rustup

  winget install -e --id ImageMagick.ImageMagick
  $magickDir = (Get-Item "C:\Program Files\ImageMagick-*").FullName

  # setx /M PATH "%PATH%;$magickDir"
  [Environment]::SetEnvironmentVariable("Path", $env:Path + ";$magickDir", "Machine")

  & "C:\\Program Files\\Git\\bin\\bash.exe" -c 'just desktop_set_tools_dev'

[windows]
desktop_set_tools_dev:
  #!pwsh
  cargo install \
    slint-lsp \
      --no-default-features \
      --features backend-winit,renderer-skia

  # Check if dprint is running
  $dprintProcess = Get-Process -Name "dprint" -ErrorAction SilentlyContinue
  if ($dprintProcess) {
    Write-Host "Running instance of dprint is detected. Stop the process (close your IDE) and press Y to continue or press N to skip" -ForegroundColor Yellow
    $response = Read-Host
    if ($response -eq 'Y' -or $response -eq 'y') {
      iwr https://dprint.dev/install.ps1 -useb | iex
    } else {
      Write-Host "Skipping dprint installation" -ForegroundColor Gray
    }
  } else {
    iwr https://dprint.dev/install.ps1 -useb | iex
  }

windows_build TARGET *ARGS:
  #!/usr/bin/env sh
  just windows build {{TARGET}} {{ARGS}}

[windows]
windows_run *ARGS:
  #!/usr/bin/env sh
  just windows run native {{ARGS}}

windows ACTION='' TARGET='native' *ARGS:
  #!/usr/bin/env sh
  OUR_PLATFORM="$(just native)"
  if [ '{{TARGET}}' = 'native' ]; then
    export TARGET="${OUR_PLATFORM}"
  else
    echo "[INFO] Cross-compilation: ${OUR_PLATFORM} >> {{TARGET}}"
    just cc_to_win_set_deps {{TARGET}} || exit 1
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
      echo "[INFO] Preparing Windows executable [${TARGET}]"
      cargo build -p feat_desktop --target ${TARGET} -r {{ARGS}}
      ls -lh ./target/${TARGET}/release/{{_app_name_windows}}* | grep '^-rwxrwxr-x'
      echo '[DONE]'
      ;;
    *)
      if [ '{{ACTION}}' = 'run' ] \
      && [ "${OUR_PLATFORM}" = "${TARGET}" ]; then
        cargo run {{ARGS}}
        exit 0
      fi

      if [ "$(just | grep 'windows_{{ACTION}}' | wc -l)" = '1' ]; then
        just windows_{{ACTION}} {{ARGS}}
      else
        echo '[INFO] Available actions:'
        just | grep windows_ | cut -c 13-
      fi
  esac
