{
  system.stateVersion = "24.05";
  networking.defaultGateway = {
    address = "192.168.1.1";
    interface = "eth0";
  };
  networking.firewall.allowedTCPPorts = [ 443 ];
  services.openssh.enable = true;
  users.users.admin = {
    isNormalUser = true;
    group = "users";
  };
}
