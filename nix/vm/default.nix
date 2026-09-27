{
  pkgs,
  nixpkgs,
  adapters,
  mkDocs,
}:
let
  inherit (pkgs) lib;
  dir = ../../tests/vms;
  bases = lib.mapAttrs' (
    f: _: lib.nameValuePair (lib.removeSuffix ".nix" f) (import (dir + "/${f}"))
  ) (builtins.readDir dir);
  options =
    (nixpkgs.lib.nixosSystem {
      modules = [ { nixpkgs.hostPlatform = pkgs.stdenv.hostPlatform.system; } ];
    }).options;
  nodesAt = import ./nodes.nix {
    inherit
      lib
      adapters
      bases
      options
      mkDocs
      ;
  };
  script = import ./script.nix { inherit lib; };
  versions = lib.unique (lib.concatMap (b: b.stateVersions) (builtins.attrValues bases));
in
lib.listToAttrs (
  map (
    sv:
    lib.nameValuePair "vm-${lib.replaceStrings [ "." ] [ "-" ] sv}" (
      pkgs.testers.runNixOSTest {
        name = "nixdiag-vm-${sv}";
        nodes = nodesAt sv;
        testScript = { nodes, ... }: script nodes;
      }
    )
  ) versions
)
