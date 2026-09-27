#
# EXAMPLE ADAPTER
#
# Maps a NixOS service onto the topology diagram.
# Copy to nix/adapters/<service>.nix.
# Hosts can change any mapped value through nixdiag.units.<service>.

{ lib, helpers }:

{
  # Label under the unit name: monitor, proxy, ... anything
  role = "monitor";

  # infra | app
  kind = "infra";

  # Required GitHub handles.
  # Monitor opens an issue for them when a nixpkgs head breaks a read.
  maintainers = [ "kucendro" ];

  # Optional. Option that turns the adapter on. Default: services.<service>.enable.
  # enable = [ "services.grafana.enable" ];

  # Values the adapter needs, by name. Each lists option paths, newest first.
  # First path present at the head wins. None present: null.
  # <name> walks an attrsOf: services.nginx.virtualHosts.<name>.forceSSL.
  reads = {
    port = [
      "services.example.settings.server.http_port"
      "services.example.port"
    ];
    domain = [ "services.example.settings.server.domain" ];
  };

  # Reads in, topology out. Takes every read by name; any can be null.
  # Returns any of: ports, names, expose, connections, scope, description.
  topology =
    { port, domain }:
    {
      ports = lib.optional (port != null) port;
      names = lib.optional (domain != null && !helpers.loopback domain) domain;
    };

  # Required VM tests -------------------------------------------------
  # Harness boots the VMs, turns the adapter on, compares its claims to what runs.
  tests = {
    # Premade VMs
    vm = [ "mon" ];

    # Minimum setup for successful bootstrap.
    # Never an option from reads: eval fails.
    # Every VM has curl and a self-signed cert: /etc/vm/cert.pem, /etc/vm/key.pem.
    minimum.services.example.settings = {
      security.secret_key = "test";
      server.http_addr = "0.0.0.0";
    };

    # Optional:
    # Values by read name, written at the first path present.
    # Exercises reads the defaults leave idle. URLs name premade VMs.
    # probe.port = 3001;
  };
}
