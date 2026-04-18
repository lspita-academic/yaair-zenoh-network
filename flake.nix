{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    # https://github.com/oxalica/rust-overlay
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        # esp-clang ships as a pre-built binary linked against libxml2.so.2
        # (the old v2.x soname). Nix's current libxml2 uses libxml2.so.16.
        # This package creates a lib directory that has both the real .so.16
        # and a .so.2 symlink pointing at it, satisfying the dynamic linker.
        # https://github.com/esp-rs/esp-idf-template/issues/282
        libxml2-16 = pkgs.libxml2.out;
        libxml2-2-links = pkgs.runCommand "libxml2-2-links" { } ''
          mkdir -p $out/lib
          ln -s "${libxml2-16}/lib/libxml2.so" $out/lib/libxml2.so.2
        '';
      in
      {
        devShell =
          with pkgs;
          mkShell {
            buildInputs = lib.lists.flatten [
              # nix
              nixd
              nil
              nixfmt
              # rust
              rust-toolchain
              rust-analyzer
              # toml
              tombi
              # esp32
              espflash
              python313 # there is a warning in the logs about 3.14
              ldproxy
            ];

            env = {
              LD_LIBRARY_PATH = lib.makeLibraryPath [
                # esp32
                stdenv.cc.cc.lib
                libxml2-2-links
                zlib
                libclang # also used by bindgen
              ];
            };

            shellHook = ''
              set -a
              source .env 2> /dev/null
              set +a
            '';
          };
      }
    );
}
