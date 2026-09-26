{ self, nixpkgs }:
let
  src = builtins.path {
    path = ../tests/fixture;
    name = "source";
  };

  flake = {
    outPath = src;
    nixosConfigurations = nixpkgs.lib.genAttrs [ "luna" "sol" ] (
      name:
      nixpkgs.lib.nixosSystem {
        modules = [
          "${src}/hosts/${name}"
          { nixpkgs.hostPlatform = "x86_64-linux"; }
        ];
      }
    );
  };
in
{
  inherit flake;

  packages =
    { pkgs, nixdiag }:
    rec {
      fixture-docs = self.lib.mkDocs {
        inherit pkgs flake;
        buildWiki = false;
        domains.ts = "ts.example";
      };

      fixture-facts = pkgs.writeText "facts.json" (builtins.toJSON (self.lib.mkFacts { inherit flake; }));

      fixture-docs-closures =
        pkgs.runCommand "nixdiag-fixture-closures"
          {
            nativeBuildInputs = [ nixdiag ];
            facts = fixture-facts;
          }
          ''
            nixdiag render --facts "$facts" --repo ${src} \
              --closures ${src}/closures.json \
              --domain ts=ts.example --out $out --no-svg
          '';
    };
}
