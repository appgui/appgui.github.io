#!/usr/bin/env sh
command -v nix &> /dev/null && {
  echo '[INFO] Nix package manager was detected. To continue, run'
  echo '  nix develop'
  exit 0
} || echo '[WARN] Nix package manager was not found.'

echo '[INFO] Attempting distro native dependency install...'
just linux_debian_set_deps \
|| just linux_fedora_set_deps \
|| just linux_opensuse_set_deps
