# NixOS module. `serve` points an nginx vhost at a docs derivation, so the
# wiki ships atomically with every deploy — no daemon, no timer. `timer` is
# the fallback for repos that want a checkout regenerated on a schedule.
{ self }:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.nixdiag;
in
{
  options.services.nixdiag = {
    serve = {
      enable = lib.mkEnableOption "serving nixdiag docs via nginx";
      docs = lib.mkOption {
        type = lib.types.package;
        description = ''
          Docs derivation, typically nixdiag.lib.mkDocs { … }.

          This option roots an nginx vhost at the derivation, so this host's
          system closure contains the docs. A docs build that measured this
          host's closure would therefore depend on itself. `closures = true`
          detects that and skips serving hosts automatically; only an explicit
          `closures = [ ... ]` naming this host reintroduces the cycle.
        '';
      };
      virtualHost = lib.mkOption {
        type = lib.types.str;
        example = "wiki.ts.example.dev";
        description = "Name of the nginx virtual host to create.";
      };
      subpath = lib.mkOption {
        type = lib.types.str;
        default = "wiki/book";
        description = "Path inside the docs derivation to use as the web root.";
      };
      virtualHostExtra = lib.mkOption {
        type = lib.types.attrs;
        default = { };
        description = "Extra nginx virtualHost settings (listenAddresses, TLS, …).";
      };
    };

    timer = {
      enable = lib.mkEnableOption "periodic nixdiag gen via a systemd timer";
      flake = lib.mkOption {
        type = lib.types.str;
        description = "Checkout directory of the flake to document.";
      };
      out = lib.mkOption {
        type = lib.types.str;
        description = "Output directory for the generated docs.";
      };
      onCalendar = lib.mkOption {
        type = lib.types.str;
        default = "daily";
        description = "systemd OnCalendar expression.";
      };
      flags = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [ ];
        description = "Extra arguments passed to nixdiag gen.";
      };
    };
  };

  config = lib.mkMerge [
    (lib.mkIf cfg.serve.enable {
      services.nginx.enable = true;
      services.nginx.virtualHosts.${cfg.serve.virtualHost} = lib.mkMerge [
        { root = "${cfg.serve.docs}/${cfg.serve.subpath}"; }
        cfg.serve.virtualHostExtra
      ];
    })
    (lib.mkIf cfg.timer.enable {
      systemd.services.nixdiag = {
        description = "nixdiag docs generation";
        serviceConfig.Type = "oneshot";
        path = [ self.packages.${pkgs.stdenv.hostPlatform.system}.nixdiag ];
        script = "nixdiag gen --flake ${lib.escapeShellArg cfg.timer.flake} --out ${lib.escapeShellArg cfg.timer.out} ${lib.escapeShellArgs cfg.timer.flags}";
      };
      systemd.timers.nixdiag = {
        wantedBy = [ "timers.target" ];
        timerConfig = {
          OnCalendar = cfg.timer.onCalendar;
          Persistent = true;
        };
      };
    })
  ];
}
