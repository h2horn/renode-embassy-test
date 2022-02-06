{
  description = "Rust environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, utils, rust-overlay, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-toolchain = pkgs.rust-bin.selectLatestNightlyWith
          (toolchain: toolchain.default.override {
            extensions = [ "rust-src" ];
            targets = [ "thumbv7em-none-eabihf" ];
          });
      in
      {
        # `nix develop`
        devShell = pkgs.mkShell {
          buildInputs = with pkgs;
            [
              rust-toolchain
              #rust-bin.nightly.latest.default #.rust-analysis
              #pkgs.rust-bin.${rustChannel}.${rustVersion}.rls
            ];
        };
      }
    );
}
