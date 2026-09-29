{ lib, helpers }:
let
  vhost = leaf: [ "services.caddy.virtualHosts.<name>.${leaf}" ];
  words = s: builtins.filter (w: builtins.isString w && w != "") (builtins.split "[ ,\t]+" s);
  site =
    address:
    let
      plain = lib.hasPrefix "http://" address;
      u = helpers.url (if lib.hasInfix "://" address then address else "https://${address}");
    in
    {
      inherit (u) host port;
      redirect = !plain && u.host != "" && u.port != 80;
    };
  upstreams =
    text:
    let
      lines = builtins.filter builtins.isList (builtins.split "reverse_proxy([^\n{]*)" text);
      tokens = lib.concatMap (m: words (builtins.head m)) lines;
      upstream =
        t:
        !(
          lib.hasSuffix ":" t
          || lib.any (p: lib.hasPrefix p t) [
            "/"
            "@"
            "*"
          ]
        );
      url =
        t:
        if lib.hasInfix "://" t then
          t
        else if lib.hasPrefix ":" t then
          "http://localhost${t}"
        else
          "http://${t}";
    in
    map url (builtins.filter upstream tokens);
in
{
  role = "proxy";
  kind = "infra";
  maintainers = [ "kucendro" ];
  reads = {
    hostName = vhost "hostName";
    aliases = vhost "serverAliases";
    listen = vhost "listenAddresses";
    extraConfig = vhost "extraConfig";
    firewall = [ "networking.firewall.enable" ];
    tcp = [ "networking.firewall.allowedTCPPorts" ];
  };

  topology =
    {
      hostName,
      aliases,
      listen,
      extraConfig,
      firewall,
      tcp,
    }:
    let
      vhosts = builtins.attrNames (if hostName == null then { } else hostName);
      sites = v: map site (words hostName.${v} ++ aliases.${v});
      primary = v: builtins.head (sites v);
      named = s: s.host != "" && !helpers.loopback s.host;
      name = v: if named (primary v) then (primary v).host else null;
      entry = v: (primary v).port;
      open = port: firewall == false || builtins.elem port (if tcp == null then [ ] else tcp);
      addrs = v: if listen.${v} == [ ] then [ "0.0.0.0" ] else listen.${v};
      scope = v: helpers.scopeOf (open (entry v)) (addrs v);
      listens = s: [ s.port ] ++ lib.optional s.redirect 80;
      label =
        v: to:
        let
          prefix =
            if name v == null then toString (entry v) else builtins.head (lib.splitString "." (name v));
        in
        "${prefix} :${toString (helpers.url to).port}";
      connectionsOf =
        v:
        map (to: {
          inherit to;
          name = name v;
          label = label v to;
          port = entry v;
          scope = scope v;
        }) (upstreams extraConfig.${v});
      exposeOf =
        v:
        lib.optional (upstreams extraConfig.${v} == [ ]) {
          name = name v;
          port = entry v;
          scope = scope v;
        };
    in
    {
      names = lib.unique (map (s: s.host) (builtins.filter named (lib.concatMap sites vhosts)));
      ports = lib.unique (lib.concatMap listens (lib.concatMap sites vhosts));
      connections = lib.concatMap connectionsOf vhosts;
      expose = lib.concatMap exposeOf vhosts;
    };

  tests = {
    vm = [ "hub" ];

    minimum.services.caddy.virtualHosts."http://plain.test".useACMEHost = null;

    probe.extraConfig."http://plain.test" = "reverse_proxy http://mon:3000";
  };
}
