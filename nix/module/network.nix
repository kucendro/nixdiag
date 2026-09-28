{ config, lib }:
let
  net = config.networking;
  over =
    kind:
    lib.mapAttrs (
      _: v: {
        inherit kind;
        over = v.interfaces;
      }
    );
  kinds =
    lib.mapAttrs (_: i: {
      kind =
        if !i.virtual then
          "physical"
        else if i.virtualType != null then
          i.virtualType
        else
          "virtual";
    }) net.interfaces
    // over "bridge" net.bridges
    // over "bond" net.bonds
    // lib.mapAttrs (_: v: {
      kind = "vlan";
      vlan = v.id;
      over = [ v.interface ];
    }) net.vlans
    // lib.mapAttrs (_: w: {
      kind = "wireguard";
      port = w.listenPort;
    }) net.wireguard.interfaces;
  enslaved =
    lib.concatMap (v: v.interfaces) (lib.attrValues net.bridges ++ lib.attrValues net.bonds)
    ++ lib.attrNames net.wireguard.interfaces;
  addressing =
    name:
    let
      i = net.interfaces.${name} or null;
      v4 = if i == null then [ ] else i.ipv4.addresses;
      v6 = if i == null then [ ] else i.ipv6.addresses;
      explicit = if i == null then null else i.useDHCP;
    in
    {
      addresses =
        map (a: "${a.address}/${toString a.prefixLength}") (v4 ++ v6)
        ++ net.wireguard.interfaces.${name}.ips or [ ];
      dhcp =
        (net.dhcpcd.enable || net.useNetworkd)
        && !builtins.elem name enslaved
        && (if explicit != null then explicit else net.useDHCP && v4 == [ ]);
    };
in
{
  interfaces = lib.mapAttrs (name: k: k // addressing name) kinds;
  gateways = map (g: { inherit (g) address interface; }) (
    builtins.filter (g: g != null) [
      net.defaultGateway
      net.defaultGateway6
    ]
  );
}
