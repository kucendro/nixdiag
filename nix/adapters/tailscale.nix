{ lib, helpers }:
{
  role = "mesh-node";
  kind = "infra";
  maintainers = [ "kucendro" ];
  reads = {
    up = [ "services.tailscale.extraUpFlags" ];
    set = [ "services.tailscale.extraSetFlags" ];
  };
  topology =
    { up, set }:
    let
      flags = (if up == null then [ ] else up) ++ (if set == null then [ ] else set);
      server = helpers.flagValue "--login-server" flags;
      routes = lib.concatMap (lib.splitString ",") (helpers.flagValues "--advertise-routes" flags);
    in
    {
      connections = [
        {
          to = if server == null then "internet" else server;
          label = "mesh";
        }
      ]
      ++ map (r: {
        to = "lan";
        label = "advertise ${r}";
      }) routes;
    };
}
