{ lib, helpers }:
let
  flags = up: set: (if up == null then [ ] else up) ++ (if set == null then [ ] else set);
  login = up: set: helpers.flagValue "--login-server" (flags up set);
  saas = "controlplane.tailscale.com";
in
{
  role = "mesh-node";
  kind = "infra";
  maintainers = [ "kucendro" ];
  reads = {
    up = [ "services.tailscale.extraUpFlags" ];
    set = [ "services.tailscale.extraSetFlags" ];
    iface = [ "services.tailscale.interfaceName" ];
  };

  topology =
    { up, set, ... }:
    let
      server = login up set;
      routes = lib.concatMap (lib.splitString ",") (
        helpers.flagValues "--advertise-routes" (flags up set)
      );
    in
    {
      connections = [
        {
          to = if server == null then "internet" else server;
          label = "mesh";
          plane = "control";
        }
      ]
      ++ map (r: {
        to = r;
        label = "advertise";
      }) routes;
    };

  networks =
    { up, set, ... }:
    lib.optionalAttrs (login up set == null) {
      tailnet = {
        cidrs = [
          "100.64.0.0/10"
          "fd7a:115c:a1e0::/48"
        ];
        kind = "mesh";
        server = saas;
      };
    };

  interfaces =
    {
      up,
      set,
      iface,
    }:
    let
      server = login up set;
    in
    lib.optionalAttrs (iface != null && iface != "userspace-networking") {
      ${iface} = {
        kind = "mesh";
        server = if server == null then saas else (helpers.url server).host;
      };
    };

  tests = {
    vm = [ "web" ];

    probe.up = [ "--login-server=http://hub:8080" ];
  };
}
