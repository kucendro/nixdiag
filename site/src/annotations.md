# Annotations

Topology is not guessed. Every node, edge and endpoint comes from a `#:` line
in your own module files. Comments are invisible to eval, so nixdiag parses
them from the repo source with rnix, in both the CLI and the derivation mode.

The grammar is frozen since 2026-08-26. New statements and optional tokens may
be added, existing ones keep their meaning.

```nix
{
  #: mesh-control
  #: name hs.example.com
  #: expose 443 public name=hs.example.com
  services.headscale.enable = true;
}
```

Sigil `#:`, two characters because you write it often. `# nixdiag:` is the
long alias, and both are recognized inside a leading `/** */` doc comment:

```nix
/**
  nginx fronts every internal service.

  #: proxy
*/
{
  services.nginx.enable = true;
}
```

One statement per line. A malformed line is a reported error, never silently
ignored.

## Attachment

Where a line sits decides what it describes.

| Position | Attaches to |
|---|---|
| directly above a `services.<x>` or `programs.<x>` binding | that service |
| in a file-leading `/** */` doc comment | whatever that file defines |
| in a host entry module (`hosts/sol/default.nix`) | the host |
| after a `#: unit <name>` line | that declared node |

A contiguous block of `#:` lines shares one attachment, so role, scope and
edges stack above a single binding. A blank line or ordinary code ends the
block.

## `#: <role>`

Role is the implicit verb, so a bare word is a role:

```nix
#: proxy
#: mesh-node
#: my-own-word
```

`mesh-control`, `proxy`, `monitor`, `dns`, `storage` and `gateway` draw as
infrastructure. `mesh-node`, `agent` and any word of your own draw as an app.
Unknown roles render with defaults, so the diagram vocabulary is yours to
extend without touching nixdiag.

## `#: expose`

```nix
#: expose 443 public name=hs.example.com
#: expose 3000 mesh
#: expose 51820/udp public
```

Port first, then an optional scope (`public`, `mesh`, `lan`) and an optional
`name=<fqdn>`. Every expose is a row on the Endpoints page. `public` and `lan`
also draw an edge to the internet or lan cloud in the topology. Without a
scope the node's `#: scope` applies.

## `#: ->` and `#: <-`

```nix
#: -> headscale hs :8080
#: -> nas/grafana metrics
#: -> hs.example.com mesh
#: -> internet
#: <- grafana scrapes
```

Target first, then an optional label drawn on the edge. `<-` reverses the
arrow. Targets:

| Target | Resolves to |
|---|---|
| `nas` | that host |
| `nas/grafana` | that service on that host |
| `grafana` | that service, on whichever host runs it |
| `hs.example.com` | the node that declared this fqdn with `#: name` |
| `internet`, `lan` | the cloud nodes |

Any enabled service is a valid target for free, because the generic
projection knows them all. References resolve against real evaluated state,
not strings, so a typo or a service you turned off is an error at render time.

### `name=` on an edge

```nix
#: -> nas/vaultwarden vault name=vault.example.com:443
```

The annotated node fronts that fqdn for the target: a proxy vhost, in
practice. It becomes an Endpoints row (scope from the node's `#: scope`, port
as written, a dash when omitted). The diagram is deliberately unaffected, the
edge is the same edge.

## `#: name`

```nix
#: name hs.example.com
```

Address book. The fqdn now resolves to this node, so other files can point an
edge at `hs.example.com` without knowing which host runs it.

## `#: scope`

```nix
#: scope mesh
```

Default scope for this node's exposes and fronted endpoints. Scopes are
`public`, `mesh` and `lan`, and they drive edge colour and the Endpoints
column.

## `#: unit`

```nix
#: unit qore
#: unit sol/nginx
```

Declares a node the module system cannot see: an OCI container, a raw
`systemd.services.*` unit, anything without a `services.<x>` binding. The
contiguous block after it attaches to that node.

```nix
{
  #: unit exporter
  #: <- grafana scrapes
  systemd.services.exporter.serviceConfig.ExecStart = "…";
}
```

In a file-leading doc comment, `unit` sets the file's default attachment.
File-level lines anywhere in that file then attach to it, which is what lets a
plain data file carry annotations next to its entries:

```nix
/**
  Upstream table for the mesh vhosts; annotations attach to nginx.

  #: unit sol/nginx
  #: scope mesh
*/
{
  #: -> luna/grafana grafana :3000 name=grafana@ts:443
  grafana = "luna.ts.example:3000";
}
```

Per-binding attachment still wins over the file default.

The `host/` prefix pins the host. It is required when several hosts reach the
same file, which happens as soon as two hosts import one table: a proxy and a
blackbox exporter sharing an endpoint list, for example. Unpinned, every
annotation in the file would be duplicated onto both hosts.

The import graph follows `imports = [ ./a.nix ]` lists and plain
`import ./a.nix` expressions alike.

## `@key` domains

Any fqdn position accepts `<sub>@<key>`, and a bare `@<key>` is the domain
itself:

```nix
#: -> nas/vaultwarden vault name=vault@home:443
#: expose 443 public name=hs@ts
```

The suffix comes from a domain map you declare:

```sh
nixdiag gen --domain home=home.example.com --domain ts=ts.example.com
```

```nix
nixdiag.domains = { home = "home.example.com"; ts = "ts.example.com"; };  # flake output
domains.home = "home.example.com";                                        # mkDocs argument
```

CLI flags override the flake. An unknown key is a hard render error, never a
literal `@home` in the output.

This exists so a public repo can document private endpoints: the source shows
`vault@home`, the rendered wiki shows `vault.home.example.com`, and the wiki
is the thing you keep on the tailnet.

## Grammar editions

Annotations live in *your* module files, so the grammar is the one nixdiag
surface that outlives nixdiag versions. It is versioned as an **edition** — a
single integer, Cargo's model but lighter — and the binary reports the one it
implements:

```console
$ nixdiag --version
nixdiag 0.1.0
annotation grammar 1
```

Declaring an edition is optional; unset means "whatever this binary
implements", so zero-config stays zero-config. Declare one when you want the
mismatch caught loudly rather than guessed at:

```nix
nixdiag.grammar = 1;          # mode A, in your flake
```

```nix
nixdiag.lib.mkDocs { grammar = 1; /* … */ }   # mode B
```

`--grammar N` overrides both, like every other setting.

- **Declared newer than the binary implements** — hard error naming both
  numbers. Upgrade nixdiag, or lower the declaration.
- **Declared older** — compatibility mode. Spellings retired since then keep
  working; nothing is ever removed *inside* an edition.

### Deprecation

A statement is never changed out from under you. When a replacement spelling
ships, the old one keeps working and the renderer warns, with the file and
line every annotation already carries:

```
warning: modules/mesh.nix:3: `#: tailnet` deprecated since 0.5, use `#: mesh`
```

Want CI red immediately instead of at the next edition? Promote the warnings:

```sh
nixdiag check --deny deprecated
```

`nixdiag.deny = [ "deprecated" ];` and `mkDocs`'s `deny` do the same. Removal
happens only at an edition bump, with an error naming the replacement — and
`nixdiag migrate --to N` rewrites the comment lines for you to review as a
diff. One statement per line is what makes that mechanical.

Nothing is deprecated in grammar 1.

## Zero annotations

Nothing breaks. The wiki, the module tree and the hosts, services and ports
pages are unaffected. The topology renders hosts with their firewall ports and
no edges, plus a hint on stderr pointing here.

## Full example

The two-host fixture that renders the diagrams on this site:

```nix
# hosts/sol/default.nix
/**
  Edge node: mesh control plane and the fleet's reverse proxy.
*/
{
  imports = [ ../../modules/mesh.nix ../../modules/web.nix ];
}

# modules/mesh.nix
{
  #: mesh-control
  #: name hs@ts
  #: expose 443 public name=hs@ts
  services.headscale.enable = true;
}

# modules/monitoring.nix
{
  #: monitor
  #: scope mesh
  #: expose 3000 name=grafana.ts.example
  services.grafana.enable = true;

  #: unit exporter
  #: <- grafana scrapes
  systemd.services.exporter.serviceConfig.ExecStart = "…";
}

# hosts/luna/default.nix
{
  #: mesh-node
  #: -> hs.ts.example mesh
  #: -> lan advertise 192.168.1.0/24
  services.tailscale.enable = true;
}
```

Rendered: [live demo](./demo.md).
