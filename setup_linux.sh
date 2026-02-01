#!/bin/sh
sudo su -c '\
  yes | apt install just rustup libssl-dev \
  || yes | dnf install just rustup openssl-devel \
'
curl -fsSL https://dprint.dev/install.sh | sh
rustup toolchain install stable
just setup_host_tools
