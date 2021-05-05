{
  description = "automerge-rs";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            overlays = [ rust-overlay.overlay ];
            inherit system;
          };
          rust = pkgs.rust-bin.nightly.latest.rust;
          cargoNix = pkgs.callPackage ./Cargo.nix { };
        in
        {

          devShell = pkgs.mkShell {
            buildInputs = with pkgs;
              [
                (rust.override {
                  extensions = [ "rust-src" ];
                })
                cargo-edit
                cargo-watch
                crate2nix

                graphviz

                rnix-lsp
                nixpkgs-fmt
              ];
          };
        });
}
