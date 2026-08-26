{
  system.stateVersion = "24.05";
  networking.firewall.allowedTCPPorts = [
    22
    443
  ];
  users.users.admin = {
    isNormalUser = true;
    group = "users";
  };
}
