/**
  nginx fronts every internal service; *mesh* vhosts bind the mesh IP.

  #: proxy
*/
{
  services.nginx = {
    enable = true;
    #: -> headscale hs :8080
    virtualHosts."hs.ts.example" = {
      locations."/".proxyPass = "http://127.0.0.1:8080";
    };
    #: -> diddy/grafana grafana :3000
    virtualHosts."grafana.ts.example" = {
      listenAddresses = [ "100.64.0.1" ];
      locations."/" = {
        proxyPass = "http://$grafana_upstream";
        extraConfig = "set $grafana_upstream diddy.ts.example:3000;";
      };
    };
  };
}
