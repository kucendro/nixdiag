# Topology

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

Adapters turn that into headscale on 8080, a proxy exposing `hs.ts.example`
on 443 to the internet, and the connection between them. Override with
`nixdiag.*` options in plain Nix.

## Adapters

| Adapter | Reads | Gives |
|---|---|---|
| `nginx` | vhost `forceSSL`, `addSSL`, `onlySSL`, `listen`, `listenAddresses`, `proxyPass`; firewall | names, ports; a literal `proxyPass` is a connection, any other vhost an expose |
| `headscale` | `port`, `settings.server_url` | port, name |
| `tailscale` | `extraUpFlags`, `extraSetFlags` | `--login-server` is a connection to it, else `internet`; `--advertise-routes` to `lan` |
| `grafana` | `settings.server.http_port`, `settings.server.domain` | port, name |

Scope from listen addresses: `100.64.0.0/10` is `mesh`, RFC 1918 is `lan`,
anything else `public` when the firewall opens the port.

## Options

`nixdiag.units.<unit>`:

| Option | Type | Effect |
|---|---|---|
| `role` | string | node label, `proxy` say |
| `kind` | `infra`, `app` | node style, adapters set `infra` |
| `scope` | `public`, `mesh`, `lan` | default for exposes and connections |
| `description` | lines | Services page section |
| `names` | fqdns | what the unit answers to |
| `ports` | ports | what it listens on |
| `expose` | `{ port, udp, scope, name }` | Endpoints row; `public` and `lan` draw the cloud edge |
| `connections` | `{ to, label, name, port, scope }` | outbound edges; `name` adds an Endpoints row |

`nixdiag.description`, `scope`, `names`, `expose`: the same for the host.

## Overriding

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

Assignment wins over an adapter, lists append, `mkForce` replaces. An empty
attrset declares a unit no adapter knows.

## Targets

| `to` | Resolves to |
|---|---|
| `internet`, `lan` | that cloud |
| `nas` | host |
| `nas/grafana` | unit on host |
| `grafana` | unit, when one host has it |
| `hs.ts.example` | unit with that fqdn in `names`, `expose` or `connections` |
| `http://127.0.0.1:8080` | unit on the same host with that port |
| `http://luna.ts.example:3000` | host `luna`, its unit on 3000 |

Unresolved fails the build.

## Adding an adapter

```nix
{ lib, helpers }:
{
  role = "monitor";
  kind = "infra";
  maintainers = [ "you" ];
  reads.port = [
    "services.foo.settings.port"
    "services.foo.port"
  ];
  topology = { port }: { ports = lib.optional (port != null) port; };
}
```

`nix/adapters/foo.nix`. First candidate present wins, a missing one reads
`null`, `<name>` walks an `attrsOf`. Enabled by `services.foo.enable` unless
`enable` says otherwise.

## Audit

```sh
nix run github:kucendro/nixdiag#audit
```

Checks every read against `options.json` of `nixos-unstable` and
`nixos-25.05`: `ok`, `renamed`, `unaudited`, `broken`. Weekly on GitHub as
the "adapter audit" issue.
