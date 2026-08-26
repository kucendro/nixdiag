host:

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

  tailscale = c.services.tailscale.enable or false;
  routes = c.services.tailscale.extraSetFlags or [ ];
  headscale = c.services.headscale.enable or false;
  headscalePort = c.services.headscale.port or 0;

  baseDomain = str' (
    c.services.headscale.settings.dns.base_domain or (c.services.headscale.settings.base_domain or "")
  );

  policyPath = str' (c.services.headscale.settings.policy.path or "");
  beszelHub = c.services.beszel.hub.enable or false;
  beszelHubPort = c.services.beszel.hub.port or 0;
  beszelAgent = c.services.beszel.agent.enable or false;
  prometheus = c.services.prometheus.enable or false;
  blackbox = c.services.prometheus.exporters.blackbox.enable or false;
  grafana = c.services.grafana.enable or false;
  promTargets = builtins.concatLists (
    map (s: builtins.concatLists (map (sc: sc.targets or [ ]) (s.static_configs or [ ]))) (
      c.services.prometheus.scrapeConfigs or [ ]
    )
  );

  vhosts = map (
    n:
    let
      v = c.services.nginx.virtualHosts.${n};
      l = v.locations."/" or { };
    in
    {
      name = n;
      listen = v.listenAddresses or [ ];
      pass = l.proxyPass or null;
      extra = (v.extraConfig or "") + (l.extraConfig or "");
    }
  ) (builtins.attrNames (c.services.nginx.virtualHosts or { }));

  services = enabledWithFiles (o.services or { });
  programs = enabledWithFiles (o.programs or { });
}
