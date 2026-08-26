{
  #: mesh-control
  #: name hs@ts
  #: expose 443 public name=hs@ts
  services.headscale = {
    enable = true;
    port = 8080;
    settings.dns.base_domain = "ts.example";
  };
}
