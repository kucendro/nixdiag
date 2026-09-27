{ lib, helpers }:
{
  role = "monitor";
  kind = "infra";
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

  # Required VM tests -------------------------------------------------
  tests = {
    vm = [ "mon" ];

    # Minimum setup for successful bootstrap
    minimum.services.grafana.settings = {
      security.secret_key = "test";
      server.http_addr = "0.0.0.0";
    };
  };
}
