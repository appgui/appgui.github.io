#!/usr/bin/env -S just --justfile
# SlintXPlatform - Project Command Runner

set dotenv-load
set export
set quiet := true

# variables
_is_linux := if os() == "linux" { "1" } else { "0" }
_is_macos := if os() == "macos" { "1" } else { "0" }
_is_windows := if os() == "windows" { "1" } else { "0" }

app_name := "slintxplatform"

# context
import? "Justpart.host.just"
import "Justpart.android.just"

_list:
  just --list

env:
  #!/bin/sh
  env

# --- SETUP ---

setup_host_deps:
  #!/bin/sh
  if [ '{{_is_linux}}' = '1' ]; then
    echo '[INFO] Setting up Linux host'
    ./setup_linux.sh
  elif [ '{{_is_macos}}' = '1' ]; then
    echo '[INFO] Setting up macOS host'
    sh ./setup_macos.sh
  elif [ '{{_is_windows}}' = '1' ]; then
    echo '[INFO] Setting up Windows host'
    powershell ./setup_windows.ps1 || pwsh ./setup_windows.ps1
  else
    echo '[FAIL] Unsupported host'
    exit 1
  fi

setup_host_tools:
  #!/bin/sh
  cargo install cargo-apk2 cargo-bundle

# --- DESKTOP ---

run_desktop:
  #!/bin/sh
  cargo run -p feat_desktop

build_desktop *ARGS: # add --release to produce optimised binary
  #!/bin/sh
  if [ '{{_is_macos}}' = '1' ]; then
    # https://github.com/burtonageo/cargo-bundle
    cargo bundle -p feat_desktop {{ARGS}}
  else
    cargo build -p feat_desktop {{ARGS}}
  fi

# --- UTILS ---

clean:
  #!/bin/sh
  cargo clean

release VARIANT:
  #!/bin/sh
  case '{{VARIANT}}':
    android)
      echo '[INFO] Preparing Android release'
      just build_android --release
      ls -lh 'target/release/apk/{{app_name}}.apk'
      ;;
    desktop)
      echo '[INFO] Preparing desktop release'
      just build_desktop --release
      ls -lh ./target/release/{{app_name}}* | grep '^-rwxrwxr-x'
      ;;
    *)
      echo '[FAIL] Unknown variant'
  esac
