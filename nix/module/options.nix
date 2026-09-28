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
  connection = types.submodule {
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
      kind = optional (
        types.enum [
          "infra"
          "app"
        ]
      );
      inherit scope;
      description = optional types.lines;
      names = list types.str;
      ports = list types.port;
      expose = list expose;
      connections = list connection;
    };
  };
  network = types.submodule {
    options = {
      cidrs = list types.str;
      kind = scope;
      server = optional types.str;
    };
  };
  interface = types.submodule {
    options = {
      kind = optional (
        types.enum [
          "physical"
          "virtual"
          "tun"
          "tap"
          "vlan"
          "bridge"
          "bond"
          "wireguard"
          "mesh"
        ]
      );
      addresses = list types.str;
      server = optional types.str;
    };
  };
in
{
  options.nixdiag = {
    description = optional types.lines;
    location = optional types.str;
    role = optional types.str;
    inherit scope;
    names = list types.str;
    expose = list expose;
    units = mkOption {
      type = types.attrsOf unit;
      default = { };
    };
    networks = mkOption {
      type = types.attrsOf network;
      default = { };
    };
    interfaces = mkOption {
      type = types.attrsOf interface;
      default = { };
    };
    facts = mkOption {
      type = types.raw;
      readOnly = true;
      internal = true;
    };
  };
}
