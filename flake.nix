{
  description = "typic: Using OCR to convert images of formulas into Typst code.";

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      imports = [
        {
          perSystem =
            { lib, ... }:
            {
              _module.args.l = lib // builtins;
            };
        }

        inputs.treefmt-nix.flakeModule
        ./rust.flakeModule.nix
        ./python.flakeModule.nix
        ./paper/flakeModule.nix
      ];
      perSystem =
        {
          l,
          pkgs,
          config,
          system,
          ...
        }:
        {
          devShells.default = pkgs.mkShell {
            inputsFrom = [
              config.devShells.rust
              config.devShells.python
              config.devShells.paper
            ];

            packages = l.attrValues {
              inherit (pkgs) just;
              inherit (inputs.oxen.packages.${system}) oxen-cli;
            };
          };

          treefmt.config = {
            projectRootFile = "flake.nix";
            programs.alejandra.enable = true;
          };
        };
    };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix.url = "github:numtide/treefmt-nix";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    typst-packages = {
      url = "github:typst/packages";
      flake = false;
    };

    oxen = {
      url = "github:oxen-ai/oxen";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
