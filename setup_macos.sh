#!/usr/bin/env sh
brew install just && \
just macos_set_deps && \
just desktop_set_tools && \
just macos_set_deps
