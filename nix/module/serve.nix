{ config, lib, ... }:
let
  cfg = config.services.nixdiag.serve;

  corsHeaders =
    if lib.length cfg.allowOrigins == 1 then
      ''
        add_header Access-Control-Allow-Origin "${lib.head cfg.allowOrigins}" always;
      ''
    else
      ''
        add_header Access-Control-Allow-Origin $nixdiag_allow_origin always;
        add_header Vary Origin always;
      '';
in
{
  options.services.nixdiag.serve = {
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
    api = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = ''
        Serve the machine-readable `api/` tree the docs derivation carries,
        at /api/. Harmless when the derivation was built with
        `mkDocs { api = false; }` — the location simply 404s.

        It exposes nothing the book at / does not already render as HTML,
        only a spelling other programs can read.
      '';
    };
    allowOrigins = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      example = [ "https://dash.example.com" ];
      description = ''
        Origins allowed to read this vhost cross-origin. Empty (the default)
        sends no CORS header at all.

        Deliberately not a wildcard option. This vhost is typically reachable
        only on a mesh or LAN, and "*" turns "reachable from my tailnet" into
        "readable by any page a browser on my tailnet happens to visit". Name
        the origins.

        A GET of a JSON file with no custom request headers is a CORS
        *simple* request, so no preflight is needed and none is generated.
        Credentials, methods and max-age belong in virtualHostExtra.

        nginx only inherits `add_header` into a location that declares none
        of its own, so a location added via virtualHostExtra with its own
        add_header will silently drop this one.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    services.nginx.enable = true;
    services.nginx.virtualHosts.${cfg.virtualHost} = lib.mkMerge [
      { root = "${cfg.docs}/${cfg.subpath}"; }
      (lib.mkIf cfg.api { locations."/api/".alias = "${cfg.docs}/api/"; })
      (lib.mkIf (cfg.allowOrigins != [ ]) { extraConfig = corsHeaders; })
      cfg.virtualHostExtra
    ];
    services.nginx.appendHttpConfig = lib.mkIf (lib.length cfg.allowOrigins > 1) ''
      map $http_origin $nixdiag_allow_origin {
          default "";
      ${lib.concatMapStrings (o: "    \"${o}\" $http_origin;\n") cfg.allowOrigins}}
    '';
    assertions = [
      {
        assertion = !(lib.any (o: lib.hasInfix "\"" o || lib.hasInfix "\n" o) cfg.allowOrigins);
        message = "services.nixdiag.serve.allowOrigins: an origin may not contain a quote or a newline; it is written verbatim into an nginx config.";
      }
    ];
  };
}
