{
  imports = [
    ../../modules/common.nix
    ../../modules/monitoring.nix
  ];

  #: mesh-node
  #: -> hs.ts.example mesh
  #: -> lan advertise 192.168.1.0/24
  services.tailscale = {
    enable = true;
    extraSetFlags = [ "--advertise-routes=192.168.1.0/24" ];
  };
}
