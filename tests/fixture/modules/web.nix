{
  services.nginx = {
    enable = true;
    virtualHosts."hs.ts.example" = {
      forceSSL = true;
      sslCertificate = "/var/lib/certs/hs.pem";
      sslCertificateKey = "/var/lib/certs/hs.key";
      locations."/".proxyPass = "http://127.0.0.1:8080";
    };
    virtualHosts."grafana.ts.example" = {
      forceSSL = true;
      sslCertificate = "/var/lib/certs/grafana.pem";
      sslCertificateKey = "/var/lib/certs/grafana.key";
      listenAddresses = [ "100.64.0.1" ];
      locations."/".proxyPass = "http://luna.ts.example:3000";
    };
  };
}
