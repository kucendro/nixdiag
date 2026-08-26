# The single generic projection — applied to a nixosConfigurations.<host> or
# darwinConfigurations.<host> value. Schema 2 reads only quasi-frozen,
# stack-agnostic surfaces; all topology semantics come from `#:` annotations
# in the documented repo, parsed at render time.
{ host, kind }:

let

  c = host.config;
  o = host.options;
  str' = v: if v == null then "" else builtins.toString v;

  enabledWithFiles =
    opts:
    builtins.filter (x: x != null) (
      map (
        n:
        let
          d = builtins.tryEval (opts.${n}.enable.definitionsWithLocations or [ ]);
          defs = if d.success then d.value else [ ];
          on = builtins.filter (e: e.value == true) defs;
        in
        if on != [ ] then
          {
            name = n;
            files = map (e: e.file) on;
          }
        else
          null
      ) (builtins.attrNames opts)
    );

in

if kind == "darwin" then
  {
    kind = "darwin";
    # x.name or x tolerates both plain-string and attrset cask entries
    casks = map (x: x.name or x) (c.homebrew.casks or [ ]);
    daemons = builtins.attrNames (c.launchd.daemons or { });
    userAgents = builtins.attrNames (c.launchd.user.agents or { });
    services = enabledWithFiles (o.services or { });
    programs = enabledWithFiles (o.programs or { });
  }
else
  {
    kind = "nixos";

    platform = str' (c.nixpkgs.hostPlatform.system or "");
    stateVersion = str' (c.system.stateVersion or "");
    tcp = c.networking.firewall.allowedTCPPorts or [ ];
    udp = c.networking.firewall.allowedUDPPorts or [ ];

    users = builtins.filter (n: c.users.users.${n}.isNormalUser or false) (
      builtins.attrNames (c.users.users or { })
    );

    pkgCount = builtins.length (c.environment.systemPackages or [ ]);

    services = enabledWithFiles (o.services or { });
    programs = enabledWithFiles (o.programs or { });
  }
