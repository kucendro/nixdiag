{ lib, helpers }:
let
  vhost = leaf: [ "services.caddy.virtualHosts.<name>.${leaf}" ];
in
{
  role = "proxy";
  kind = "infra";
  maintainers = [ "kucendro" ];
  reads = {
    extraConfig = vhost "extraConfig";
    firewall = [ "networking.firewall.enable" ];
    tcp = [ "networking.firewall.allowedTCPPorts" ];
  };

  topology =
    {
      extraConfig,
      firewall,
      tcp,
    }:
    let
      configs = if extraConfig == null then { } else extraConfig;
      names = builtins.attrNames configs;
      plain = name: lib.hasPrefix "http://" name;
      portsOf =
        name:
        if plain name then
          [ 80 ]
        else
          [
            80
            443
          ];
      entry = name: if plain name then 80 else 443;
      open = port: firewall == false || builtins.elem port (if tcp == null then [ ] else tcp);
      scope = name: helpers.scopeOf (open (entry name)) [ "0.0.0.0" ];
      targets = name: builtins.split "reverse_proxy[ \t]+([^ \t\n{]+)" configs.${name};
      upstreams =
        name:
        lib.concatMap (
          m: lib.optional (builtins.isList m && !lib.hasInfix "{" (builtins.head m)) (builtins.head m)
        ) (targets name);
      label = name: builtins.head (lib.splitString "." (lib.removePrefix "http://" name));
      connectionsOf =
        name:
        map (to: {
          inherit to name;
          label = "${label name} :${toString (helpers.url to).port}";
          port = entry name;
          scope = scope name;
        }) (upstreams name);
      exposeOf =
        name:
        lib.optional (upstreams name == [ ]) {
          inherit name;
          port = entry name;
          scope = scope name;
        };
    in
    {
      inherit names;
      ports = lib.unique (lib.concatMap portsOf names);
      connections = lib.concatMap connectionsOf names;
      expose = lib.concatMap exposeOf names;
    };

  # Required VM tests -------------------------------------------------
  tests = {
    vm = [ "web" ];

    # Minimum setup for successful bootstrap
    minimum.services.caddy.virtualHosts."http://plain.test".serverAliases = [ ];

    # Values for reads the defaults leave idle
    probe = {
      extraConfig."http://plain.test" = "reverse_proxy mon:3000";
      tcp = [ 80 ];
    };
  };
}
