{ config, lib, ... }:
let
  adapters = import ../adapters { inherit lib; };

  walk =
    node: path:
    if path == [ ] then
      node
    else if !(builtins.isAttrs node) then
      null
    else
      let
        seg = builtins.head path;
        rest = builtins.tail path;
      in
      if seg == "<name>" then
        lib.mapAttrs (_: v: walk v rest) node
      else if node ? ${seg} then
        walk node.${seg} rest
      else
        null;

  read =
    path:
    let
      v = walk config (lib.splitString "." path);
      r = builtins.tryEval (builtins.deepSeq v v);
    in
    if r.success then r.value else null;

  readAny = candidates: lib.findFirst (v: v != null) null (map read candidates);

  scalars = [
    "role"
    "kind"
    "scope"
  ];

  enabled = name: adapter: readAny (adapter.enable or [ "services.${name}.enable" ]) == true;

  values = adapter: lib.mapAttrs (_: readAny) adapter.reads;

  unitOf =
    name: adapter:
    lib.mkIf (enabled name adapter) (
      lib.mapAttrs (k: v: if builtins.elem k scalars then lib.mkDefault v else v) (
        {
          inherit (adapter) role;
          kind = adapter.kind or null;
        }
        // adapter.topology (values adapter)
      )
    );

  declared =
    key:
    lib.mkMerge (
      lib.mapAttrsToList (
        name: adapter: lib.mkIf (adapter ? ${key} && enabled name adapter) (adapter.${key} (values adapter))
      ) adapters
    );
in
{
  config.nixdiag = {
    units = lib.mapAttrs unitOf adapters;
    networks = declared "networks";
    interfaces = declared "interfaces";
  };
}
