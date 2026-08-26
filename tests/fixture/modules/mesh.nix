{
  services.headscale = {
    enable = true;
    port = 8080;
    settings.dns.base_domain = "ts.example";
  };
}
