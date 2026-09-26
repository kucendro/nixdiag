{
  services.grafana = {
    enable = true;
    settings.server.http_port = 3000;
  };

  systemd.services.exporter = {
    serviceConfig.ExecStart = "/run/current-system/sw/bin/true";
  };

  nixdiag.units.grafana = {
    description = "Dashboards over the exporter's metrics.";
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
