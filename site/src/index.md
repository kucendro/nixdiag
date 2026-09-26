# nixdiag

Static infrastructure docs from any Nix flake: topology, module tree, flake
inputs and an mdBook wiki from `nixosConfigurations` and
`darwinConfigurations`.

```sh
nix run github:kucendro/nixdiag
```

Topology comes from the NixOS module system. Adapters read the options you
already set, `nixdiag.*` options override them.

![Data-flow topology](./topology.svg)

![Module tree](./modules.svg)

Both are the two-host fixture, the [live demo](./demo.md).

## What you get

| File | Contents |
|---|---|
| `topology.d2`, `topology.svg` | who talks to what, by scope: public, mesh, lan |
| `modules.d2`, `modules.svg` | host to module file tree |
| `inputs.d2`, `inputs.svg` | flake input graph; `follows` edges dashed |
| `wiki/src/index.md` | your hand-written overview, written once, never overwritten |
| `wiki/src/architecture.md` | both diagrams |
| `wiki/src/hosts.md` | per host: platform, users, ports, services and their files |
| `wiki/src/services.md` | every service, the hosts running it, the file defining it |
| `wiki/src/endpoints.md` | fqdn, port, scope, host, service |
| `wiki/src/inputs.md`, `inputs-timeline.svg` | every input with its rev and lock date, a timeline of those dates, plus duplicate detection |
| `wiki/src/closures.md`, `closures.svg` | opt-in: per-host closure size, largest paths, fleet sharing, stacked bar chart |

## Next

- [Quickstart](./quickstart.md)
- [Topology](./topology.md): adapters, `nixdiag.*` options, overrides
- [Build and serve](./build.md): `mkDocs`, the nginx module
- [CLI](./cli.md): `render`
