{
  description = "Static infrastructure docs from any Nix flake";

  inputs.nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      eachSystem = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      fixtureSrc = builtins.path {
        path = ./tests/fixture;
        name = "source";
      };

      fixtureFlake = {
        outPath = fixtureSrc;
        nixosConfigurations = nixpkgs.lib.genAttrs [ "diddy" "epstein" ] (
          name:
          nixpkgs.lib.nixosSystem {
            modules = [
              "${fixtureSrc}/hosts/${name}"
              { nixpkgs.hostPlatform = "x86_64-linux"; }
            ];
          }
        );
      };
    in
    {
      lib = import ./nix/lib.nix {
        inherit self;
        lib = nixpkgs.lib;
      };

      packages = eachSystem (pkgs: rec {
        default = nixdiag;

        fixture-docs = self.lib.mkDocs {
          inherit pkgs;
          flake = fixtureFlake;
          buildWiki = false;
        };

        nixdiag = pkgs.rustPlatform.buildRustPackage {
          pname = "nixdiag";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postInstall = ''
            wrapProgram $out/bin/nixdiag \
              --suffix PATH : ${
                nixpkgs.lib.makeBinPath [
                  pkgs.d2
                  pkgs.nix
                ]
              }
          '';

          meta = {
            description = "Static infrastructure docs from any Nix flake";
            license = nixpkgs.lib.licenses.mit;
            mainProgram = "nixdiag";
          };
        };
      });

      overlays.default = final: prev: {
        nixdiag = self.packages.${final.stdenv.hostPlatform.system}.nixdiag;
      };

      nixosModules.default = import ./nix/module.nix { inherit self; };

      templates.default = {
        path = ./templates/default;
        description = "Flake with nixdiag docs as a pure derivation";
      };

      apps = eachSystem (pkgs: {
        default = {
          type = "app";
          program = nixpkgs.lib.getExe self.packages.${pkgs.stdenv.hostPlatform.system}.nixdiag;
        };
      });

      devShells = eachSystem (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
            d2
            mdbook
            lefthook
            nixfmt-rfc-style
          ];
        };
      });

      checks = eachSystem (pkgs: {
        build = self.packages.${pkgs.stdenv.hostPlatform.system}.nixdiag;
        golden =
          pkgs.runCommand "nixdiag-golden"
            {
              docs = self.packages.${pkgs.stdenv.hostPlatform.system}.fixture-docs;
              golden = ./tests/golden;
            }
            ''
              diff -u "$golden/topology.d2" "$docs/topology.d2"
              diff -u "$golden/modules.d2" "$docs/modules.d2"
              diff -u "$golden/hosts.md" "$docs/wiki/src/hosts.md"
              touch $out
            '';
      });
    };
}
