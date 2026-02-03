#!/usr/bin/env -S just --justfile

_is_amd64 := shell('if [ "$(uname -m)" = "x86_64" ]; then echo "1"; else echo "0"; fi')

appimagetool_version := "1.9.1"

appimagetool_url_amd64 := join('https://github.com/AppImage/appimagetool/releases/download', appimagetool_version, 'appimagetool-x86_64.AppImage')
appimagetool_url_aarch64 := join('https://github.com/AppImage/appimagetool/releases/download', appimagetool_version, 'appimagetool-aarch64.AppImage')

setup_appimagetool:
  #!/bin/sh
  set -eu

  cd ./.tools
  echo '[INFO] Downloading appimagetool {{ appimagetool_version }}...'

  if [ '{{_is_amd64}}' = '1' ]; then
    appimagetool_dl_link='{{ appimagetool_url_amd64 }}'
  else
    appimagetool_dl_link='{{ appimagetool_url_aarch64 }}'
  fi

  wget -O ./appimagetool "$appimagetool_dl_link" || \
    curl -L -o ./appimagetool "$appimagetool_dl_link"

  chmod +x ./appimagetool
