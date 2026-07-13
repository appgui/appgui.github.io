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
import? "Justpart.android.just"
import? "Justpart.ios.just"

# web
import? "Justpart.web.just"

_list:
  just --list

_workspace_set:
  #!/usr/bin/env sh
  mkdir -p ./target 2> /dev/null \
  || true
  just _desktop_set
  just _android_set
  # just _ios_set
  # just _web_set

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
  cargo fmt --all # essential formatting (Rust code)

  if command -v slint-lsp > /dev/null \
  && command -v dprint > /dev/null; then
    :
  else
    echo '[INFO] Installing linters...'
    just desktop_set_tools_dev
  fi

  slint-lsp format -i ui/**/*.slint
  dprint fmt
