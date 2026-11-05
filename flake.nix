{
  description = "appGUI dev env";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        fenix-pkgs = fenix.packages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Core tools
            cargo
            dprint
            fontconfig
            imagemagick
            just
            ninja
            unzip

            # Fenix respects rust-toolchain.toml
            fenix-pkgs.complete.toolchain

            # OpenSSL/libssl development libraries
            openssl
            pkg-config

            # Wayland graphics
            wayland
            libxkbcommon
            libGL

            # Platform-specific additions
          ] ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
            # macOS-specific (Darwin) dependencies
            darwin.apple_sdk.frameworks.CoreServices
          ] ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
            # Linux-specific dependencies
            glibc
          ];

          # Set up environment variables for OpenSSL detection
          shellHook = ''
            # Mimique the tmpfiles rule: `systemd.tmpfiles.rules = [ "d /run/user/%U/just 0755 %u users -" ];`
            mkdir -m 0755 -p /run/user/$(id -u)/just
            mkdir -m 0755 -p ~/.rustup/downloads

            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [
              pkgs.openssl
              pkgs.wayland
              pkgs.libxkbcommon
              pkgs.libGL
            ]}:$LD_LIBRARY_PATH
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

            # For rustup/cargo builds
            export OPENSSL_DIR="${pkgs.openssl.dev}"
            export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
            export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"

            export PATH="$HOME/.cargo/bin:$PATH"
          '';
        };
      }
    );
}
