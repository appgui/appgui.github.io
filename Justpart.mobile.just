#!/usr/bin/env -S just --justfile

_mobile_set:
  #!/usr/bin/env sh
  cd ./feat_mobile
  # if [ ! -f ./Cargo.toml ]; then
    cp -f ./Cargo.base.toml ./Cargo.toml
  # fi

  if [ ! -f ./target ] && [ ! -L ./target ]; then
    ln -sf ../target ./target
  fi
  if [ ! -f ./assets ] && [ ! -L ./assets ]; then
    ln -sf ../assets ./assets
  fi
