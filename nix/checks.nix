{
  pkgs,
  packages,
  self,
  nixpkgs,
}:
let
  snapshots = ../tests/reference;

  mkClosures =
    (import ./closures.nix {
      inherit pkgs;
      lib = pkgs.lib;
    }).mkClosures;

  check =
    name: env:
    pkgs.runCommand "nixdiag-${name}"
      (
        {
          nativeBuildInputs = [
            pkgs.jq
            (pkgs.callPackage ./d2.nix { })
          ];
        }
        // env
      )
      ''
        bash ${../tests/checks}/${name}.sh
        touch $out
      '';
in
{
  build = packages.nixdiag;
  site = packages.site;

  reference = check "reference" {
    docs = packages.fixture-docs;
    reference = snapshots;
  };

  closures = check "closures" {
    docs = packages.fixture-docs-closures;
    reference = snapshots;
  };

  assets = check "assets" {
    fresh = packages.fixture-assets;
    assets = ../assets;
  };

  closures-plumbing = check "closures-plumbing" {
    closures = mkClosures { toplevels.demo = pkgs.hello; };
  };
  closures-self = check "closures-self" {
    closures = mkClosures { toplevels.nixdiag = packages.nixdiag; };
  };
}
// pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux (
  import ./vm {
    inherit pkgs nixpkgs;
    inherit (self.lib) adapters mkDocs;
  }
)
