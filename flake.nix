{
  description = "Rust environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    mach-nix.url = "mach-nix/3.4.0";
  };

  outputs = { self, nixpkgs, utils, rust-overlay, mach-nix, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
          (self: super: {
            renode = super.callPackage ./renode.nix {};
          })
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-toolchain = pkgs.rust-bin.selectLatestNightlyWith
          (toolchain: toolchain.default.override {
            extensions = [ "rust-src" ];
            targets = [ "thumbv7em-none-eabihf" "thumbv7em-none-eabi" ];
          });
        renode-python = mach-nix.lib.${system}.mkPython {
          requirements = builtins.readFile "${pkgs.renode}/opt/renode/tests/requirements.txt";
        };
        renode-test-script = pkgs.writeScriptBin "renode-test" ''
          python -u ${pkgs.renode}/opt/renode/tests/run_tests.py \
            --robot-framework-remote-server-full-directory=${pkgs.renode}/bin \
            --robot-framework-remote-server-name=renode \
            --css-file=${pkgs.renode}/opt/renode/tests/robot.css \
            --variable PWD_PATH:"$PWD" \
            --runner=none \
            -r test-results $@
        '';
      in
      {
        # `nix develop`
        devShell = pkgs.mkShell {
          buildInputs = with pkgs;
            [
              rust-toolchain
              renode
              gcc-arm-embedded # gdb
              renode-python
              renode-test-script
            ];
            RENODE_PATH = "${pkgs.renode}/opt/renode/";
        };
      }
    );
}
