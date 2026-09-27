{ mkDocs }:
{
  config,
  options,
  extendModules,
  nodes,
  pkgs,
  lib,
  ...
}:
{
  options.vm.self = lib.mkOption {
    type = lib.types.raw;
    internal = true;
  };

  config = {
    vm.self = { inherit config options extendModules; };
    services.nixdiag.serve.docs = mkDocs {
      inherit pkgs;
      flake = {
        outPath = pkgs.writeTextDir "flake.nix" "{ }";
        nixosConfigurations = lib.mapAttrs (_: n: n.vm.self) nodes;
      };
      closures = true;
    };
  };
}
