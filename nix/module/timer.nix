{ self }:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.nixdiag.timer;
in
{
  options.services.nixdiag.timer = {
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

  config = lib.mkIf cfg.enable {
    systemd.services.nixdiag = {
      description = "nixdiag docs generation";
      serviceConfig.Type = "oneshot";
      path = [ self.packages.${pkgs.stdenv.hostPlatform.system}.nixdiag ];
      script = "nixdiag gen --flake ${lib.escapeShellArg cfg.flake} --out ${lib.escapeShellArg cfg.out} ${lib.escapeShellArgs cfg.flags}";
    };
    systemd.timers.nixdiag = {
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnCalendar = cfg.onCalendar;
        Persistent = true;
      };
    };
  };
}
