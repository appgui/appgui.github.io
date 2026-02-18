#!/usr/bin/env -S just --justfile
# xPlatform - Project Command Runner

set dotenv-load
set export
set shell := ["sh", "-c"]
set windows-shell := ["C:\\Program Files\\Git\\usr\\bin\\sh.exe", "-cu"]
set quiet := true

# desktop
import "Justpart.desktop.just"
import? "Justpart.desktop.linux.just"
import? "Justpart.desktop.macos.just"
import? "Justpart.desktop.windows.just"

# mobile
import "Justpart.mobile.just"
import? "Justpart.mobile.android.just"
import? "Justpart.mobile.ios.just"

_list:
  just --list

_workspace_set:
  #!/usr/bin/env sh
  mkdir -p ./target 2> /dev/null || true
  just _desktop_set
  just _mobile_set

env:
  #!/usr/bin/env sh
  env

native:
  #!/usr/bin/env sh
  rustc -Vv | grep 'host: ' | cut -d' ' -f2

clean:
  #!/usr/bin/env sh
  just _workspace_set
  cargo clean

format:
  #!/usr/bin/env sh
  cargo fmt --all

release VARIANT:
  #!/usr/bin/env sh
  just {{VARIANT}} build --release
  # case '{{VARIANT}}':
  #   desktop)
  #   linux)
  #   macos)
  #   windows)
  #   mobile)
  #   android)
  #   ios)
  #     ;;
  #   *)
  #     echo '[FAIL] Unknown variant'
  # esac
