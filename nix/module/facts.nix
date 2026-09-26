{ config, options, ... }:
let
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

  common = {
    services = enabledWithFiles (options.services or { });
    programs = enabledWithFiles (options.programs or { });
    inherit (config.nixdiag) description;
    topology = {
      inherit (config.nixdiag)
        role
        scope
        names
        expose
        units
        ;
    };
  };

  darwin = {
    kind = "darwin";
    casks = map (x: x.name or x) (config.homebrew.casks or [ ]);
    daemons = builtins.attrNames (config.launchd.daemons or { });
    userAgents = builtins.attrNames (config.launchd.user.agents or { });
  };

  nixos = {
    kind = "nixos";
    platform = str' (config.nixpkgs.hostPlatform.system or "");
    stateVersion = str' (config.system.stateVersion or "");
    tcp = config.networking.firewall.allowedTCPPorts or [ ];
    udp = config.networking.firewall.allowedUDPPorts or [ ];
    users = builtins.filter (n: config.users.users.${n}.isNormalUser or false) (
      builtins.attrNames (config.users.users or { })
    );
    pkgCount = builtins.length (config.environment.systemPackages or [ ]);
  };
in
{
  imports = [
    ./options.nix
    ./adapters.nix
  ];

  config.nixdiag.facts = (if options ? launchd then darwin else nixos) // common;
}
