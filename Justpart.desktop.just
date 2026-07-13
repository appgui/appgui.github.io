#!/usr/bin/env -S just --justfile

_is_linux := if os() == "linux" { "1" } else { "0" }
_is_macos := if os() == "macos" { "1" } else { "0" }
_is_windows := if os() == "windows" { "1" } else { "0" }

_desktop_set:
  #!/usr/bin/env sh
  cd ./feat_desktop

  if [ ! -d ./target ] \
  && [ ! -L ./target ]; then
    ln -sf ../target ./target
  fi
  if [ ! -d ./assets ] \
  && [ ! -L ./assets ]; then
    ln -sf ../assets ./assets
  fi

desktop_icons_make:
  #!/usr/bin/env sh
  cd ./assets
  magick -background transparent -density 1200 ./icon.svg \
    -define icon:auto-resize="256,128,96,64,48,32,16" \
      ./icon.ico

desktop_set_deps:
  #!/usr/bin/env sh
  if [ '{{_is_linux}}' = '1' ]; then
    echo '[INFO] Setting up Linux host'
    ./setup_linux.sh
  elif [ '{{_is_macos}}' = '1' ]; then
    echo '[INFO] Setting up macOS host'
    sh ./setup_macos.sh
  elif [ '{{_is_windows}}' = '1' ]; then
    echo '[INFO] Setting up Windows host'
    pwsh ./setup_windows.ps1 || /c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe ./setup_windows.ps1
  else
    echo '[FAIL] Unsupported host'
    exit 1
  fi

desktop_build *ARGS:
  #!/usr/bin/env sh
  just desktop build native {{ARGS}}

desktop_run *ARGS:
  #!/usr/bin/env sh
  just desktop run native {{ARGS}}

desktop ACTION='' TARGET='native' *ARGS:
  #!/usr/bin/env sh
  just _workspace_set
  if [ ! -f ./assets/icon.ico ]; then
    just desktop_icons_make
  fi

  cd ./feat_desktop

  case '{{ACTION}}' in
    build)
      set -e

      if [ -f ../.env ]; then
        . ../.env
      fi || echo '[INFO] Not using compat mode'

      # export PATH="$(pwd)/.tools:$PATH"

      if [ '{{_is_macos}}' = '1' ]; then
        just macos build {{TARGET}} {{ARGS}} \
        && exit 0
      elif [ '{{_is_linux}}' = '1' ]; then
        just linux build {{TARGET}} {{ARGS}} \
        && exit 0
      fi

      just windows build {{TARGET}} {{ARGS}}
      ;;
    run)
      cargo run {{ARGS}}
      ;;
    *)
      if [ "$(just | grep 'desktop_{{ACTION}}' | wc -l)" = '1' ]; then
        just desktop_{{ACTION}} {{ARGS}}
      else
        echo '[INFO] Available actions:'
        just | grep desktop_ | cut -c 13-
        echo 'run                     # native'
        echo 'build'
      fi
  esac
