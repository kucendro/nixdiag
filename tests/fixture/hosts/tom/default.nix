{
  imports = [
    ../../modules/common.nix
    ../../modules/monitoring.nix
  ];

  services.tailscale = {
    enable = true;
    extraUpFlags = [ "--login-server=https://hs.ts.example" ];
    extraSetFlags = [ "--advertise-routes=192.168.1.0/24" ];
  };
}
