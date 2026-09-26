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
    pkgs.runCommand "nixdiag-${name}" ({ nativeBuildInputs = [ pkgs.jq ]; } // env) ''
      bash ${../tests/checks}/${name}.sh
      touch $out
    '';

  serving = rec {
    flake = {
      outPath = ../tests/fixture;
      nixosConfigurations.web = nixpkgs.lib.nixosSystem {
        modules = [
          ../nix/module
          {
            nixpkgs.hostPlatform = pkgs.stdenv.hostPlatform.system;
            system.stateVersion = "25.05";
            fileSystems."/" = {
              device = "/dev/sda";
              fsType = "ext4";
            };
            boot.loader.grub.device = "/dev/sda";
            services.nixdiag.serve = {
              enable = true;
              inherit docs;
              virtualHost = "docs.example";
            };
          }
        ];
      };
    };
    docs = self.lib.mkDocs {
      inherit pkgs flake;
      buildWiki = false;
      closures = [ "web" ];
    };
  };
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

  closures-plumbing = check "closures-plumbing" {
    closures = mkClosures { toplevels.demo = pkgs.hello; };
  };
  closures-self = check "closures-self" {
    closures = mkClosures { toplevels.nixdiag = packages.nixdiag; };
  };
}
// pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
  closures-serving =
    pkgs.runCommand "nixdiag-closures-serving"
      { drv = builtins.unsafeDiscardStringContext serving.docs.drvPath; }
      ''
        echo "$drv" > $out
      '';
}
