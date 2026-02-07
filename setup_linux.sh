#!/usr/bin/env sh
sudo su -c '\
  yes | apt install just rustup libssl-dev \
  || yes | dnf install just rustup openssl-devel \
'
curl -fsSL https://dprint.dev/install.sh | sh
rustup toolchain install stable
just desktop_set_tools
just linux_debian_set_deps || just linux_fedora_set_deps
