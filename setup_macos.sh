#!/bin/sh
brew install dprint just ninja && \
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
rustup toolchain install stable
just setup_host_tools
