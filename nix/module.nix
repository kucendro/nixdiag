{ config, lib, ... }:
let
  cfg = config.services.nixdiag.serve;
in
{
  options.services.nixdiag.serve = {
    enable = lib.mkEnableOption "serving nixdiag docs via nginx";
    docs = lib.mkOption {
      type = lib.types.package;
      description = ''
        Docs derivation, typically nixdiag.lib.mkDocs { … }.
        Rooting a vhost here puts the docs into this host's closure, so
        `closures = true` skips this host; only an explicit `closures = [ … ]`
        naming it reintroduces the cycle.
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

  config = lib.mkIf cfg.enable {
    services.nginx.enable = true;
    services.nginx.virtualHosts.${cfg.virtualHost} = lib.mkMerge [
      { root = "${cfg.docs}/${cfg.subpath}"; }
      cfg.virtualHostExtra
    ];
  };
}
