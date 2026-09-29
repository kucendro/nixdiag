{ lib, ... }:
{
  role = "remote-access";
  kind = "infra";
  maintainers = [ "kucendro" ];
  reads = {
    ports = [ "services.openssh.ports" ];
    listen = [ "services.openssh.listenAddresses" ];
  };

  topology =
    { ports, listen }:
    let
      base = if ports == null then [ 22 ] else ports;
      each = l: if l.port == null then base else [ l.port ];
    in
    {
      ports = lib.unique (if listen == null || listen == [ ] then base else lib.concatMap each listen);
    };

  tests = {
    vm = [ "mon" ];

    probe.listen = [
      {
        addr = "0.0.0.0";
        port = 2222;
      }
    ];
  };
}
