{
  description = "Functional programming language with algebraic effects";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:nixos/nixpkgs";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        # overlays = [ fenix.overlays.default ];
        # pkgs = import nixpkgs {
        # inherit system overlays;
        # };
        rustToolchainFile = (pkgs.lib.importTOML ./rust-toolchain.toml).toolchain;
        rustToolchain = (
          inputs'.fenix.packages.fromToolchainName {
            name = rustToolchainFile.channel;
            sha256 = "sha256-AJ6LX/Q/Er9kS15bn9iflkUwcgYqRQxiOIL2ToVAXaU=";
          }
        );
        rust = rustToolchain.toolchain;
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs =
            [
              rust
            ]
            ++ (with pkgs; [
              bacon
              gdb
            ]);
        };

        formatter = pkgs.alejandra;
      };
      flake = {};
    };
}
