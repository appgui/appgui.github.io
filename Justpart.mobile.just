#!/usr/bin/env -S just --justfile

_mobile_set:
  #!/usr/bin/env sh
  cd ./feat_mobile
  # if [ ! -f ./Cargo.toml ]; then
    cp -f ./Cargo.base.toml ./Cargo.toml
  # fi

  if [ ! -d ./target ] && [ ! -L ./target ]; then
    ln -sf ../target ./target
  fi
  if [ ! -d ./res ]; then
    just android_icons_make
  fi
