{
  #: monitor
  #: scope mesh
  #: expose 3000 name=grafana.ts.example
  services.grafana = {
    enable = true;
    settings.server.http_port = 3000;
  };
}
