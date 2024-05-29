{
  description = "Functional programming language with algebraic effects";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust = fenix.packages.${system}.default.toolchain;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
          ] ++ (with pkgs; [
            rust-analyzer
            bacon
            gdb
          ]);
        };
      }
    );
}
