# Topology

Every node, connection and endpoint is the value of a `nixdiag.*` option in
your NixOS configuration. Adapters fill those options from the options you
already set; you override them with plain Nix.

```nix
{
  services.headscale = {
    enable = true;
    port = 8080;
    settings.server_url = "https://hs.ts.example";
  };
  services.nginx.virtualHosts."hs.ts.example" = {
    forceSSL = true;
    locations."/".proxyPass = "http://127.0.0.1:8080";
  };
}
```

No nixdiag line in there, and the diagram shows headscale on port 8080,
nginx as a proxy exposing `hs.ts.example` on 443 to the internet, and a
connection labelled `hs :8080` from the proxy into headscale. Rendered in
full as the [live demo](./demo.md).

`mkFacts` evaluates each configuration with the nixdiag module added through
`extendModules`, or uses your copy when the flake imports
`nixosModules.default`. A connection that cannot be resolved fails
`nix build .#docs`.

## Adapters

One file per service under `nix/adapters/`, each a list of the options it
reads and a function from their values to topology. A unit appears when its
service is enabled.

| Adapter | Reads | Gives |
|---|---|---|
| `nginx` | per vhost `forceSSL`, `addSSL`, `onlySSL`, `listen`, `listenAddresses`, `locations.*.proxyPass`; `defaultListenAddresses`; the firewall | `names` from vhosts, `ports` from listens. A literal `proxyPass` is a connection labelled `<vhost> :<upstream port>`, any other vhost an expose |
| `headscale` | `port`, `settings.server_url` or `serverUrl` | `ports`, the url's host as a name unless loopback |
| `tailscale` | `extraUpFlags`, `extraSetFlags` | `--login-server=URL` is a connection labelled `mesh` to that url, otherwise to `internet`; each `--advertise-routes` a connection to `lan` |
| `grafana` | `settings.server.http_port` or `port`, `settings.server.domain` | `ports`, the domain as a name unless loopback |

Scope comes from listen addresses: `100.64.0.0/10` and `fd7a:115c:a1e0::/48`
are `mesh`, RFC 1918 and `fc00::/7` are `lan`, loopback has none, and anything
else is `public` when the firewall opens the port. All four adapters draw as
infrastructure.

## Options

Per unit, under `nixdiag.units.<unit>`:

| Option | Type | Effect |
|---|---|---|
| `role` | string | second line of the node label, `proxy` say |
| `kind` | `infra` or `app` | node style; adapters set `infra`, a unit of yours is `app` |
| `scope` | `public`, `mesh`, `lan` | default for the unit's exposes and connections |
| `description` | lines | a section on the Services page |
| `names` | list of fqdn | what this unit answers to; connection targets resolve against them |
| `ports` | list of port | what it listens on; a url target resolves to the unit on its port |
| `expose` | list of `{ port, udp, scope, name }` | an Endpoints row each; `public` and `lan` draw an edge from that cloud |
| `connections` | list of `{ to, label, name, port, scope }` | outbound edges; `name` makes an Endpoints row this unit fronts |

Per host, under `nixdiag`: `description` for the Hosts page, `scope`,
`names` and `expose` as above, on the host itself.

## Overriding

Adapters set `role`, `kind` and `scope` with `mkDefault`, lists at normal
priority. A plain assignment wins, list entries append, `mkForce` replaces.

```nix
{ lib, ... }:
{
  nixdiag.units.nginx.role = "gateway";
  nixdiag.units.grafana = {
    scope = "mesh";
    expose = [ { port = 3000; name = "grafana.ts.example"; } ];
    connections = [ { to = "exporter"; label = "scrapes"; } ];
  };
  nixdiag.units.tailscale.connections = lib.mkForce [ ];
  nixdiag.units.exporter = { };
}
```

`exporter` is a raw systemd unit no adapter knows; the empty attrset declares
it as a node.

## Targets

| `to` | Resolves to |
|---|---|
| `internet`, `lan` | that cloud |
| `nas` | that host |
| `nas/grafana` | that unit on that host |
| `grafana` | that unit, when exactly one host has it |
| `hs.ts.example` | the unit whose `names`, `expose` or `connections` carry that fqdn |
| `http://127.0.0.1:8080` | the unit on the same host with 8080 in `ports` or `expose` |
| `http://luna.ts.example:3000` | the host named by the first label, then its unit on 3000 |

## Adding an adapter

```nix
{ lib, helpers }:
{
  role = "monitor";
  kind = "infra";
  maintainers = [ "you" ];
  reads = {
    port = [
      "services.foo.settings.port"
      "services.foo.port"
    ];
  };
  topology = { port }: { ports = lib.optional (port != null) port; };
}
```

Drop it in as `nix/adapters/foo.nix`. Each read is a list of candidate option
paths, first present wins, so the list is also the rename history. A `<name>`
segment walks an `attrsOf` and hands the function an attrset per name. Reads
are forced under `tryEval`, so a missing or throwing option reads as `null`.
`enable` defaults to `services.foo.enable`; set `enable = [ "programs.foo.enable" ]`
when the predicate lives elsewhere.

## Audit

```sh
nix run github:kucendro/nixdiag#audit
```

Fetches `options.json` for `nixos-unstable` and `nixos-25.05`, without
evaluating anything, and prints one row per read: `ok`, `renamed` when a
later candidate matched, `unaudited` under a freeform `settings` option,
`broken` when none exists. Channels are positional arguments. A weekly
workflow files the table as the "adapter audit" issue and fails on `broken`.

## Zero topology

The topology renders hosts with their firewall ports and no edges, plus a
note on stderr pointing here. Every other page is unaffected.
