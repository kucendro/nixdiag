{
  #: mesh-control
  #: name hs.ts.example
  #: expose 443 public name=hs.ts.example
  services.headscale = {
    enable = true;
    port = 8080;
    settings.dns.base_domain = "ts.example";
  };
}
