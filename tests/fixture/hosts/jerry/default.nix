{
  imports = [
    ../../modules/common.nix
    ../../modules/mesh.nix
    ../../modules/web.nix
  ];

  networking.interfaces.eth0.ipv4.addresses = [
    {
      address = "192.168.1.10";
      prefixLength = 24;
    }
  ];
  networking.vlans.iot = {
    id = 20;
    interface = "eth0";
  };
  networking.interfaces.iot.ipv4.addresses = [
    {
      address = "192.168.20.1";
      prefixLength = 24;
    }
  ];
}
