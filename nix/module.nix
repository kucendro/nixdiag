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

  # Mirrors `API_VERSION` in the binary. The URL prefix is the API's version,
  # so a v2 can be served beside v1 rather than replacing it.
  apiVersion = "v1";
  historyDir = "/var/lib/nixdiag/history";

  corsHeaders =
    if lib.length cfg.serve.allowOrigins == 1 then
      ''
        add_header Access-Control-Allow-Origin "${lib.head cfg.serve.allowOrigins}" always;
      ''
    else
      ''
        add_header Access-Control-Allow-Origin $nixdiag_allow_origin always;
        add_header Vary Origin always;
      '';
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

      history = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Keep every deployed revision's api/v1/snapshot.json under
          /var/lib/nixdiag/history and serve them at /api/v1/history/.

          This is the one piece of nixdiag that holds mutable state, and it
          has to live here: a derivation is immutable, so it cannot accumulate
          across deploys. It is a oneshot run on activation — no daemon, no
          timer, no database — and nixdiag itself never reads it back.

          Snapshots are filed by revision, so a build with no revision (see
          mkDocs `revision`) is skipped rather than overwriting anything.
        '';
      };

      historyLimit = lib.mkOption {
        type = lib.types.nullOr lib.types.ints.positive;
        default = null;
        example = 200;
        description = ''
          Keep at most this many snapshots, dropping the oldest first. Null
          keeps everything, which is fine until it is not: a long-lived host
          deploying often should set a bound rather than fill /var.
        '';
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
      # Projections must never read an option here that names the docs
      # derivation — `root`, and equally `locations.*.alias`. Reading one
      # forces the derivation, and eval recurses on a host that serves the
      # docs describing it. They read vhost names, listen addresses and
      # proxyPass only; keep it that way.
      services.nginx.virtualHosts.${cfg.serve.virtualHost} = lib.mkMerge [
        { root = "${cfg.serve.docs}/${cfg.serve.subpath}"; }
        (lib.mkIf cfg.serve.api {
          locations."/api/".alias = "${cfg.serve.docs}/api/";
        })
        (lib.mkIf (cfg.serve.allowOrigins != [ ]) { extraConfig = corsHeaders; })
        (lib.mkIf cfg.serve.history {
          # The mutable half, under the same URL namespace as the immutable
          # one but living on a completely different filesystem path.
          locations."/api/${apiVersion}/history/".alias = "${historyDir}/";
        })
        cfg.serve.virtualHostExtra
      ];

      assertions = [
        {
          assertion = !(lib.any (o: lib.hasInfix "\"" o || lib.hasInfix "\n" o) cfg.serve.allowOrigins);
          message = "services.nixdiag.serve.allowOrigins: an origin may not contain a quote or a newline; it is written verbatim into an nginx config.";
        }
        {
          assertion = cfg.serve.history -> cfg.serve.api;
          message = "services.nixdiag.serve.history needs serve.api = true; the snapshots are served under /api/.";
        }
      ];
    })
    (lib.mkIf (cfg.serve.enable && lib.length cfg.serve.allowOrigins > 1) {
      # nginx omits a header whose value is empty, which is what lets one
      # `add_header` serve several origins. `Vary: Origin` is not optional
      # here: without it a cache in front hands one origin's response to
      # another.
      services.nginx.appendHttpConfig = ''
        map $http_origin $nixdiag_allow_origin {
            default "";
        ${lib.concatMapStrings (o: "    \"${o}\" $http_origin;\n") cfg.serve.allowOrigins}}
      '';
    })
    (lib.mkIf (cfg.serve.enable && cfg.serve.history) {
      systemd.services.nixdiag-history = {
        description = "File this generation's nixdiag snapshot";
        wantedBy = [ "multi-user.target" ];
        # Ahead of nginx, so a fresh deploy never serves an index that does
        # not yet list the snapshot sitting beside it.
        before = [ "nginx.service" ];
        path = [
          pkgs.jq
          pkgs.coreutils
          pkgs.findutils
        ];
        serviceConfig = {
          Type = "oneshot";
          RemainAfterExit = true;
          StateDirectory = "nixdiag/history";
          StateDirectoryMode = "0755";
        };
        script = ''
          set -euo pipefail
          snap=${cfg.serve.docs}/api/${apiVersion}/snapshot.json
          if [ ! -f "$snap" ]; then
            echo "nixdiag: these docs carry no API tree; nothing to file." >&2
            exit 0
          fi
          rev=$(jq -r '.revision.id // empty' "$snap")
          if [ -z "$rev" ]; then
            echo "nixdiag: this build carries no revision; nothing to file." >&2
            exit 0
          fi
          # Idempotent: re-activating this generation, or rolling back to it,
          # rewrites the same file and disturbs nothing else.
          install -m444 "$snap" "$STATE_DIRECTORY/$rev.json"
          ${lib.optionalString (cfg.serve.historyLimit != null) ''
            ls -1t "$STATE_DIRECTORY"/*.json 2>/dev/null \
              | tail -n +${toString (cfg.serve.historyLimit + 1)} \
              | xargs -r rm -f
          ''}
          # Rebuild the index from the snapshots only — a plain *.json glob
          # would sweep in index.json itself and append a null revision to it.
          # `sort -z` keeps the input order stable so snapshots sharing a
          # timestamp do not reshuffle between runs. Written via rename
          # because nginx may be serving the old index right now.
          find "$STATE_DIRECTORY" -maxdepth 1 -name '*.json' ! -name index.json -print0 \
            | sort -z \
            | xargs -0 -r jq -s 'map(.revision) | map(select(. != null))
                   | sort_by(.time) | reverse
                   | { meta: { generator: "Auto-generated by nixdiag. Do not edit.",
                               schema: 1 },
                       revisions: . }' > "$STATE_DIRECTORY/.index.new"
          mv "$STATE_DIRECTORY/.index.new" "$STATE_DIRECTORY/index.json"
        '';
      };
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
