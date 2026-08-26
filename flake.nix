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
        nixosConfigurations = nixpkgs.lib.genAttrs [ "luna" "sol" ] (
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
          domains.ts = "ts.example";
        };

        # The fixture's wiki, published under /demo on the docs site.
        demo-docs = self.lib.mkDocs {
          inherit pkgs;
          flake = fixtureFlake;
          title = "Example fleet";
          domains.ts = "ts.example";
        };

        # Hand-written book in site/, with the demo wiki nested under /demo
        # and the README diagrams doubling as the book's images.
        site =
          pkgs.runCommand "nixdiag-site"
            {
              nativeBuildInputs = [ pkgs.mdbook ];
            }
            ''
              cp -r ${./site} book
              chmod -R u+w book
              cp ${./assets}/topology.svg ${./assets}/modules.svg book/src/
              # --dest-dir resolves relative paths against the cwd, not the book root
              mdbook build book --dest-dir $out
              cp -r --no-preserve=mode ${demo-docs}/wiki/book $out/demo
            '';

        nixdiag = pkgs.callPackage ./nix/package.nix { };
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
        site = self.packages.${pkgs.stdenv.hostPlatform.system}.site;
        reference =
          pkgs.runCommand "nixdiag-reference"
            {
              docs = self.packages.${pkgs.stdenv.hostPlatform.system}.fixture-docs;
              reference = ./tests/reference;
            }
            ''
              diff -u "$reference/topology.d2" "$docs/topology.d2"
              diff -u "$reference/modules.d2" "$docs/modules.d2"
              diff -u "$reference/hosts.md" "$docs/wiki/src/hosts.md"
              diff -u "$reference/endpoints.md" "$docs/wiki/src/endpoints.md"
              touch $out
            '';
      });
    };
}
