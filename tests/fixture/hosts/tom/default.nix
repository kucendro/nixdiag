{
  imports = [
    ../../modules/common.nix
    ../../modules/monitoring.nix
  ];

  networking.interfaces.eth0.ipv4.addresses = [
    {
      address = "192.168.1.20";
      prefixLength = 24;
    }
  ];

  nixdiag.networks.lan.cidrs = [ "192.168.1.0/24" ];

  services.tailscale = {
    enable = true;
    extraUpFlags = [ "--login-server=https://hs.ts.example" ];
    extraSetFlags = [ "--advertise-routes=192.168.1.0/24" ];
  };
}
