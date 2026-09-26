{
  services.headscale = {
    enable = true;
    port = 8080;
    settings.server_url = "https://hs.ts.example";
    settings.dns.base_domain = "ts.example";
  };
}
