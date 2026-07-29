{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        inherit (pkgs) lib;

        # We're going to build python wheels as well, so we need a python
        python = pkgs.python3;

        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p: p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
        );
        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
          env = {
            PYO3_PYTHON = python.interpreter;
            PYO3_BUILD_EXTENSION_MODULE = true;
          };
          nativeBuildInputs = with pkgs;
            [python]
            ++ lib.optionals stdenv.isDarwin [libiconv];
          buildInputs = [];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        nkscan-cli = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            doCheck = false; # handled by nextest
            cargoExtraArgs = "--features=cli";
          }
        );
      in {
        # Checks for nix flake check
        checks = {
          nkscan = nkscan-cli;

          clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          crate-fmt = craneLib.cargoFmt {
            inherit src;
          };

          toml-fmt = craneLib.taploFmt {
            src = pkgs.lib.sources.sourceFilesBySuffices src [".toml"];
          };

          nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );
        };
        # Devshell
        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages = with pkgs; [
            cargo-outdated
          ];
        };

        # Flake entrypoint
        apps.default = flake-utils.lib.mkApp {
          drv = nkscan-cli;
        };

        # Package output
        packages = {
          default = nkscan-cli;
        };
      }
    );
}
