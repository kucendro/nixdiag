{
  services.grafana = {
    enable = true;
    settings.server.http_port = 3000;
  };

  systemd.services.exporter = {
    serviceConfig.ExecStart = "/run/current-system/sw/bin/true";
  };

  nixdiag.units.grafana = {
    scope = "mesh";
    expose = [
      {
        port = 3000;
        name = "grafana.ts.example";
      }
    ];
    connections = [
      {
        to = "exporter";
        label = "scrapes";
      }
    ];
  };
  nixdiag.units.exporter = { };
}
