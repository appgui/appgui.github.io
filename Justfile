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
import? "Justpart.linux.just"
import "Justpart.android.just"

_list:
  just --list

env:
  #!/bin/sh
  env

prepare_icons:
  #!/bin/sh
  cd ./assets
  magick -background transparent -density 384 ./icon.svg \
    -define icon:auto-resize="256,128,96,64,48,32,16" \
      ./icon.ico
  magick ./icon.svg -resize 512x512 ./icon.png

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
  cargo install cargo-appimage cargo-apk2 cargo-bundle

# --- DESKTOP ---

run_desktop:
  #!/bin/sh
  if [ ! -f ./icon.png ]; then
    just prepare_icons
  fi
  cargo run -p feat_desktop

build_desktop *ARGS: # add --release to produce optimised binary
  #!/bin/sh
  if [ ! -f ./icon.png ]; then
    just prepare_icons
  fi

  export PATH="$(pwd)/.tools:$PATH"

  if [ '{{_is_macos}}' = '1' ]; then
    echo '[INFO] Attempting to build app bundle...'
    # https://github.com/burtonageo/cargo-bundle
    cargo bundle -p feat_desktop {{ARGS}} && echo '[DONE]' && exit 0
  elif [ '{{_is_linux}}' = '1' ] && [ -f .tools/appimagetool ]; then
    # https://github.com/AppImage/appimagetool
    echo '[INFO] Attempting to build appimage...'

    # https://github.com/StratusFearMe21/cargo-appimage
    # cargo appimage -p feat_desktop && echo '[DONE]' && exit 0

    # https://github.com/burtonageo/cargo-bundle
    cargo bundle -p feat_desktop {{ARGS}} && echo '[DONE]' && exit 0
  fi

  echo '[INFO] Compiling binary...'
  cargo build -p feat_desktop {{ARGS}}
  echo '[DONE]'

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
