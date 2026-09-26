{ lib, helpers }:
{
  role = "mesh-control";
  kind = "infra";
  maintainers = [ "kucendro" ];
  reads = {
    port = [ "services.headscale.port" ];
    url = [
      "services.headscale.settings.server_url"
      "services.headscale.serverUrl"
    ];
  };
  topology =
    { port, url }:
    let
      host = if url == null then null else (helpers.url url).host;
    in
    {
      ports = lib.optional (port != null) port;
      names = lib.optional (host != null && !helpers.loopback host) host;
    };
}
