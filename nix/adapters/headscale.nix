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
  tests = {
    vm = [ "hub" ];
    minimum.services.headscale = {
      address = "0.0.0.0";
      settings.dns = {
        base_domain = "mesh.test";
        override_local_dns = false;
      };
      settings.derp = {
        urls = [ ];
        server = {
          enabled = true;
          region_id = 999;
          stun_listen_addr = "0.0.0.0:3478";
        };
      };
    };
    probe.url = "http://hub:8080";
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
