/**
  nginx fronts every internal service; *tailnet* vhosts bind the mesh IP.
*/
{
  services.nginx = {
    enable = true;
    virtualHosts."hs.ts.example" = {
      locations."/".proxyPass = "http://127.0.0.1:8080";
    };
    virtualHosts."grafana.ts.example" = {
      listenAddresses = [ "100.64.0.1" ];
      locations."/" = {
        proxyPass = "http://$grafana_upstream";
        extraConfig = "set $grafana_upstream diddy.ts.example:3000;";
      };
    };
  };
}
