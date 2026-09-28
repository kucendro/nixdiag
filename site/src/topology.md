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
| `headscale` | `port`, `settings.server_url`, `settings.prefixes` | port; name and tailnet from a non-loopback `server_url`, the tailnet named after it |
| `tailscale` | `extraUpFlags`, `extraSetFlags`, `interfaceName` | `--login-server` is a control connection to it, else `internet`; the interface on that server's tailnet, routing `--advertise-routes` and, with `--advertise-exit-node`, the internet |
| `grafana` | `settings.server.http_port`, `settings.server.domain` | port, name |

Scope from listen addresses: `100.64.0.0/10` is `mesh`, RFC 1918 is `lan`,
anything else `public` when the firewall opens the port.

## Options

`nixdiag.units.<unit>`:

| Option | Type | Effect |
|---|---|---|
| `role` | string | node label, `proxy` say |
| `kind` | `infra`, `app` | node style, adapters set `infra` |
| `scope` | `public`, `mesh`, `lan` | default for exposes and connections; unset on an expose, the firewall decides |
| `description` | lines | Services page section |
| `names` | fqdns | what the unit answers to |
| `ports` | ports | what it listens on |
| `expose` | `{ port, udp, scope, name }` | Endpoints row; an edge from the internet, or from the host's `lan` or `mesh` networks |
| `connections` | `{ to, label, name, port, scope, plane }` | outbound edges; `name` adds an Endpoints row |

`nixdiag.description`, `scope`, `names`, `expose`: the same for the host.

`plane` is `data` (default), `control` or `mgmt`: control is infrastructure
coordinating itself, a mesh node talking to its server; mgmt is you operating
it, scrapes, backups, deploys. The overview draws only data; host boards draw
every plane, control and mgmt thin and dashed.

## Networks

Every static interface address puts its host on the subnet it belongs to.
Name a subnet, or join more under one name, on any host:

```nix
nixdiag.networks.lan.cidrs = [ "192.168.1.0/24" "fd00:1::/64" ];
```

| Option | Type | Effect |
|---|---|---|
| `nixdiag.networks.<name>.cidrs` | CIDRs | addresses inside join this network |
| `nixdiag.networks.<name>.kind` | `public`, `mesh`, `lan` | cloud style; else from the addresses |
| `nixdiag.networks.<name>.server` | host | control server; a `mesh` interface with it joins |
| `nixdiag.interfaces.<name>` | `{ kind, addresses, routes, server }` | an interface NixOS does not configure; `routes` are prefixes it carries, `0.0.0.0/0` makes an exit node |

A network belongs to the `nixdiag.location` of the hosts on it, so the same
range at two locations is two networks, `192.168.1.0/24 @ home` and
`192.168.1.0/24 @ lab`. A declared name splits the same way into `lan @ home`
and `lan @ lab`; a `mesh` network never splits. A host without a location
joins the only copy of its range, and fails the build when there are several.

The same name declared differently at one location, or two declared networks
that overlap without sitting at two different locations, fails the build.

The overlay board draws every `mesh` network: its members by interface, the
routes they carry, and the control server their `control` connections reach.
Any mesh shows up once an adapter or your config declares the network, the
member interfaces and the control connection; nothing on the board is
specific to tailscale.

## Locations

```nix
nixdiag.location = "home";
```

Hosts sharing a `lan` or `public` network share a location: name it on one
host and the rest follow. A host's own `nixdiag.location` wins; two names in
one group leave its unnamed hosts out. The overview and the networks board
draw each location as a box around its hosts and the networks inside it;
mesh networks and the internet stay outside.

## Firewall

The firewall page lists, per host, what `networking.firewall` opens on every
interface, on each one, and trusts, then what does not add up:

- a port open that no unit, port or expose uses
- an expose closed on every interface its scope reaches
- a trusted interface other than `lo`: every port on it is open
- a connection from another host to a port closed on every interface they share

A port counts as used only when nixdiag knows it: a service with no adapter
and no `nixdiag.units` entry shows as unused. Raw nftables or iptables rules
are not read.

An expose with no `scope` is reached from the networks behind every interface
the firewall opens its port on, and takes the widest of their kinds.

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
| `internet` | that cloud |
| `nas` | host |
| `nas/grafana` | unit on host |
| `grafana` | unit, when one host has it |
| `hs.ts.example` | unit with that fqdn in `names`, `expose` or `connections` |
| `http://127.0.0.1:8080` | unit on the same host with that port |
| `http://tom.ts.example:3000` | host `tom`, its unit on 3000 |
| `lan` | network with that name, after hosts and units |
| `192.168.1.0/24` | network holding it, a new one if none |
| `http://192.168.1.20:3000` | host with that address, its unit on 3000; else its network |

A network, CIDR or address found at several locations resolves at the
source host's location; from a host with no location it fails the build.

Unresolved fails the build. `lan` is no longer built in: declare
`nixdiag.networks.lan.cidrs` and `to = "lan"` keeps working.

## Adding an adapter

Copy `nix/adapters/example.nix` to `nix/adapters/foo.nix`. Every field is
explained there. Besides `topology`, an adapter may define `networks` and
`interfaces`: reads in, `nixdiag.networks` and `nixdiag.interfaces` out. `tests` is required: `nix flake check` boots the VMs it names
and compares the adapter's claims to what runs.

## Audit

```sh
nix run github:kucendro/nixdiag#audit -- nixos-unstable github:NixOS/nixpkgs/master
```

Checks every read against `options.json`, downloaded for a channel or built
for any nixpkgs ref: `ok`, `renamed`, `unaudited`, `broken`; `--json` for
rows. Every day at 20:00 Prague time the monitor workflow runs it against
`master`, `nixos-unstable` and `nixos-26.05`, boots the adapter VMs on both
channels, evaluates them on `master`, and records both on the `data` branch.
An adapter that is not `ok` gets an
issue mentioning its `maintainers`, which are GitHub handles; it closes once
every head is `ok` again.
