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
| `topology.d2`, `topology.svg` | fleet overview: hosts as tables of units and ports, ingress from Internet, LAN and mesh, flows between hosts |
| `topology-<host>.d2`, `topology-<host>.svg` | per host: its units, local flows, and the hosts it talks to |
| `modules-<host>.d2`, `modules-<host>.svg` | per host: entry file to the modules it imports, each listing its units |
| `inputs.d2`, `inputs.svg` | flake input graph; `follows` edges dashed |
| `theme.d2` | classes and palette every `.d2` imports |
| `wiki/src/index.md` | a stub for your own overview, replaced by `indexPage` |
| `wiki/src/architecture.md` | topology diagram |
| `wiki/src/hosts.md` | per host: platform, users, ports, services and their files, topology, module tree |
| `wiki/src/services.md` | every service, the hosts running it, the file defining it |
| `wiki/src/endpoints.md` | fqdn, port, scope, host, service |
| `wiki/src/inputs.md`, `inputs-timeline.svg` | every input with its rev and lock date, how many days each trails the newest, plus duplicate detection |
| `wiki/src/closures.md`, `closures.svg` | opt-in: per-host closure size, largest paths, fleet sharing, stacked bar chart |

## Next

- [Quickstart](./quickstart.md)
- [Topology](./topology.md): adapters, `nixdiag.*` options, overrides
- [Build and serve](./build.md): `mkDocs`, the nginx module
- [CLI](./cli.md): flags
