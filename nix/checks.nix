{ pkgs, packages }:
let
  snapshots = ../tests/reference;

  mkClosures =
    (import ./closures.nix {
      inherit pkgs;
      lib = pkgs.lib;
    }).mkClosures;

  check =
    name: env:
    pkgs.runCommand "nixdiag-${name}" ({ nativeBuildInputs = [ pkgs.jq ]; } // env) ''
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

  closures-plumbing = check "closures-plumbing" { closures = mkClosures { demo = pkgs.hello; }; };
  closures-self = check "closures-self" { closures = mkClosures { nixdiag = packages.nixdiag; }; };
}
