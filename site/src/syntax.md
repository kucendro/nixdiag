# nixdiag annotation cheat sheet

Annotation grammar 1. This is the short form, meant to be handed to a coding
assistant; `nixdiag syntax` prints it. The full reference is
<https://kucendro.github.io/nixdiag/annotations.html>.

Topology comes from `#:` comment lines in your own Nix modules. Evaluation
never sees them; nixdiag parses the source. `# nixdiag:` is the long alias,
and both work inside a file-leading `/** */` doc comment.

## Rules

- One statement per line, on a line of its own. Plain `#` comments are free.
- Contiguous `#:` lines form one block. A blank line or code ends it.
- A malformed line, an edge to a typo or a disabled service, or an unknown
  `@key` fails the render. Nothing is skipped silently.
- Check with `nixdiag check --flake .`, or `nix build .#docs` when the docs
  are a derivation.

## Statements

| Line | Meaning |
|---|---|
| `#: proxy` | the node's role, one word. `mesh-control`, `proxy`, `monitor`, `dns`, `storage` and `gateway` draw as infrastructure; `mesh-node`, `agent` or any word of yours draw as an app |
| `#: expose 443 public name=hs@ts` | a port this node listens on, then an optional scope and an optional fqdn. Every expose is an Endpoints row; `public` and `lan` also draw an edge to that cloud |
| `#: expose 51820/udp public` | `/udp` for UDP; TCP is the default |
| `#: expose 3000 mesh` | scope is `public`, `mesh` or `lan`; without one the node's `#: scope` applies |
| `#: -> nas/grafana metrics` | an edge from this node to a target, then an optional label |
| `#: <- grafana scrapes` | the same edge with the arrow reversed |
| `#: -> nas/gitea git :3001 name=git@home:443` | `name=` on an edge: this node fronts that fqdn for the target, a proxy vhost. The `:443` is optional |
| `#: name hs@ts` | this node answers to that fqdn, so other files can target `hs@ts` |
| `#: scope mesh` | default scope for this node's exposes and fronted endpoints |
| `#: unit qore` | declares a node evaluation cannot see, such as a container or a raw systemd unit. The block attaches to it |
| `#: unit nas/qore` | the same, pinned to one host. Required once several hosts import the file |

Edge targets:

| Target | Resolves to |
|---|---|
| `nas` | that host |
| `nas/grafana` | that service or declared unit on that host |
| `grafana` | that service or unit, when exactly one host has it |
| `hs@ts` | the node that declared this fqdn with `#: name` |
| `internet`, `lan` | the cloud nodes |

`@key` works in any fqdn position: `git@home` renders as `git.` plus the domain
mapped to `home`, and a bare `@home` is that domain itself. The map comes from
the flake's `nixdiag.domains`, from `mkDocs { domains = ...; }` or from
`--domain home=example.com`, so a public repo never spells out a private
domain.

## Where a block attaches

| Block position | Attaches to |
|---|---|
| directly above a binding under `services.nginx`, or inside the `services.nginx = { ... }` set | nginx |
| the same for `programs.<x>` | that program |
| a block containing `#: unit exporter` | the declared node `exporter` |
| the file-leading `/** */` doc comment | whatever the file defines |
| that doc comment with `#: unit sol/nginx` in it | `sol/nginx` becomes the file's default, so blocks above plain bindings attach to it |
| a host entry module such as `hosts/luna/default.nix` | the host |

Per-binding attachment wins over the file default. A sub-service such as
`services.beszel.agent` folds into its parent unit unless a `#: unit` splits
it out.

## Example

Trimmed from the two-host fixture behind the live demo, rendered with
`--domain ts=ts.example`:

```nix
# modules/mesh.nix, imported by hosts/sol/default.nix
{
  #: mesh-control
  #: name hs@ts
  #: expose 443 public name=hs@ts
  services.headscale.enable = true;
}

# modules/web.nix, imported by hosts/sol/default.nix
/**
  #: proxy
*/
{
  services.nginx = {
    enable = true;
    #: -> headscale hs :8080
    virtualHosts."hs.ts.example".locations."/".proxyPass = "http://127.0.0.1:8080";
  };
}

# modules/upstreams.nix, a plain data file imported by web.nix
/**
  #: unit sol/nginx
  #: scope mesh
*/
{
  #: -> luna/grafana grafana :3000 name=grafana@ts:443
  grafana = "luna.ts.example:3000";
}

# modules/monitoring.nix, imported by hosts/luna/default.nix
{
  #: monitor
  #: scope mesh
  #: expose 3000 name=grafana@ts
  services.grafana.enable = true;

  #: unit exporter
  #: <- grafana scrapes
  systemd.services.exporter.serviceConfig.ExecStart = "/run/current-system/sw/bin/true";
}

# hosts/luna/default.nix
{
  #: mesh-node
  #: -> hs@ts mesh
  #: -> lan advertise 192.168.1.0/24
  services.tailscale.enable = true;
}
```

Result: <https://kucendro.github.io/nixdiag/demo/>.
