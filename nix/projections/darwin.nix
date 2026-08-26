# Projection applied to a darwinConfigurations.<host> value.
host:
let
  c = host.config;
  o = host.options;
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
{
  kind = "darwin";
  # x.name or x tolerates both plain-string and attrset cask entries
  casks = map (x: x.name or x) (c.homebrew.casks or [ ]);
  daemons = builtins.attrNames (c.launchd.daemons or { });
  userAgents = builtins.attrNames (c.launchd.user.agents or { });
  services = enabledWithFiles (o.services or { });
  programs = enabledWithFiles (o.programs or { });
}
