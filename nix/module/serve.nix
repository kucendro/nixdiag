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
        Closure metrics measure this host with an empty directory in its
        place, so a build may document the host that serves it.
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
