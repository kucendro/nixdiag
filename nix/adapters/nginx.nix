{ lib, helpers }:
let
  vhost = leaf: [ "services.nginx.virtualHosts.<name>.${leaf}" ];
in
{
  role = "proxy";
  kind = "infra";
  maintainers = [ "kucendro" ];
  reads = {
    forceSSL = vhost "forceSSL";
    addSSL = vhost "addSSL";
    onlySSL = vhost "onlySSL";
    listen = vhost "listen";
    listenAddresses = vhost "listenAddresses";
    defaultListenAddresses = [ "services.nginx.defaultListenAddresses" ];
    proxyPass = vhost "locations.<name>.proxyPass";
    firewall = [ "networking.firewall.enable" ];
    tcp = [ "networking.firewall.allowedTCPPorts" ];
  };
  topology =
    {
      forceSSL,
      addSSL,
      onlySSL,
      listen,
      listenAddresses,
      defaultListenAddresses,
      proxyPass,
      firewall,
      tcp,
    }:
    let
      addrs =
        name:
        if listen.${name} != [ ] then
          map (l: l.addr) listen.${name}
        else if listenAddresses.${name} != [ ] then
          listenAddresses.${name}
        else if defaultListenAddresses != null then
          defaultListenAddresses
        else
          [ "0.0.0.0" ];
      open = port: firewall == false || builtins.elem port (if tcp == null then [ ] else tcp);
      names = builtins.attrNames (if listen == null then { } else listen);
      ssl = name: forceSSL.${name} || addSSL.${name} || onlySSL.${name};
      listenPort =
        l:
        if l.port != null then
          l.port
        else if l.ssl then
          443
        else
          80;
      portsOf =
        name:
        if listen.${name} != [ ] then
          lib.unique (map listenPort listen.${name})
        else
          lib.optional (!onlySSL.${name}) 80 ++ lib.optional (ssl name) 443;
      entry = name: if ssl name then 443 else builtins.head (portsOf name);
      scope = name: helpers.scopeOf (open (entry name)) (addrs name);
      literal = builtins.filter (p: p != null && !lib.hasInfix "$" p);
      upstreams = name: literal (builtins.attrValues proxyPass.${name});
      connectionsOf =
        name:
        map (to: {
          inherit to name;
          label = "${builtins.head (lib.splitString "." name)} :${toString (helpers.url to).port}";
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
}
