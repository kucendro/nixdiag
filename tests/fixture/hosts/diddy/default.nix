{
  imports = [
    ../../modules/common.nix
    ../../modules/monitoring.nix
  ];

  services.tailscale = {
    enable = true;
    extraSetFlags = [ "--advertise-routes=192.168.1.0/24" ];
  };
}
