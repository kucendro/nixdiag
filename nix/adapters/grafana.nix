{ lib, helpers }:
{
  role = "monitor";
  maintainers = [ "kucendro" ];
  reads = {
    port = [
      "services.grafana.settings.server.http_port"
      "services.grafana.port"
    ];
    domain = [ "services.grafana.settings.server.domain" ];
  };
  topology =
    { port, domain }:
    {
      ports = lib.optional (port != null) port;
      names = lib.optional (domain != null && !helpers.loopback domain) domain;
    };
}
