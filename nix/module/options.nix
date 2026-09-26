{ lib, ... }:
let
  inherit (lib) mkOption types;
  optional =
    type:
    mkOption {
      type = types.nullOr type;
      default = null;
    };
  list =
    type:
    mkOption {
      type = types.listOf type;
      default = [ ];
    };
  scope = optional (
    types.enum [
      "public"
      "mesh"
      "lan"
    ]
  );
  expose = types.submodule {
    options = {
      port = mkOption { type = types.port; };
      udp = mkOption {
        type = types.bool;
        default = false;
      };
      inherit scope;
      name = optional types.str;
    };
  };
  edge = types.submodule {
    options = {
      to = mkOption { type = types.str; };
      label = mkOption {
        type = types.str;
        default = "";
      };
      name = optional types.str;
      port = optional types.port;
      inherit scope;
    };
  };
  unit = types.submodule {
    options = {
      role = optional types.str;
      inherit scope;
      description = optional types.lines;
      names = list types.str;
      ports = list types.port;
      expose = list expose;
      edges = list edge;
    };
  };
in
{
  options.nixdiag = {
    description = optional types.lines;
    role = optional types.str;
    inherit scope;
    names = list types.str;
    expose = list expose;
    units = mkOption {
      type = types.attrsOf unit;
      default = { };
    };
    facts = mkOption {
      type = types.raw;
      readOnly = true;
      internal = true;
    };
  };
}
