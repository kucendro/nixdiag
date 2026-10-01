{ self, nixpkgs }:
let
  src = builtins.path {
    path = ../tests/fixture;
    name = "source";
  };

  flake = {
    outPath = src;
    nixosConfigurations = nixpkgs.lib.genAttrs [ "tom" "jerry" ] (
      name:
      nixpkgs.lib.nixosSystem {
        modules = [
          "${src}/hosts/${name}"
          { nixpkgs.hostPlatform = "x86_64-linux"; }
        ]
        ++ nixpkgs.lib.optional (name == "tom") ../nix/module;
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
        generated = 1790000000;
        revision = "0123abc";
      };

      fixture-facts = pkgs.writeText "facts.json" (builtins.toJSON (self.lib.mkFacts { inherit flake; }));

      fixture-docs-closures =
        pkgs.runCommand "nixdiag-fixture-closures"
          {
            nativeBuildInputs = [ nixdiag ];
            facts = fixture-facts;
          }
          ''
            nixdiag --facts "$facts" --repo ${src} \
              --closures ${src}/closures.json --out $out --no-svg
          '';

      fixture-assets =
        pkgs.runCommand "nixdiag-fixture-assets"
          {
            nativeBuildInputs = [ nixdiag ];
            facts = fixture-facts;
          }
          ''
            mkdir $out
            for theme in dark light; do
              nixdiag --facts "$facts" --repo ${src} \
                --closures ${src}/closures.json --theme $theme --out $theme >/dev/null
              suffix=""
              [ $theme = dark ] || suffix=-light
              for f in topology modules-jerry inputs wiki/src/inputs-timeline wiki/src/closures wiki/src/closures-jerry; do
                cp $theme/$f.svg $out/$(basename $f)$suffix.svg
              done
            done
          '';
    };
}
