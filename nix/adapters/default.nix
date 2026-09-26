{ lib }:
let
  helpers = import ./lib.nix { inherit lib; };
  skip = [
    "default.nix"
    "lib.nix"
  ];
  files = builtins.filter (n: lib.hasSuffix ".nix" n && !(builtins.elem n skip)) (
    builtins.attrNames (builtins.readDir ./.)
  );
in
lib.listToAttrs (
  map (
    f: lib.nameValuePair (lib.removeSuffix ".nix" f) (import (./. + "/${f}") { inherit lib helpers; })
  ) files
)
