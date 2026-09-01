# nixdiag

Static infrastructure docs from any Nix flake. nixdiag reads your
`nixosConfigurations` and `darwinConfigurations` and renders a data-flow
topology diagram, a module tree and an mdBook wiki.

```sh
nix run github:kucendro/nixdiag -- gen --flake .
```

The topology comes from `#:` comments in your own module files, so it records
intent ("this is my proxy, it fronts grafana") instead of guessing from option
shapes that nixpkgs reshapes every month. nixdiag has no service adapters and
no built-in knowledge of tailscale, nginx or prometheus.

![Data-flow topology](./topology.svg)

The module tree shows how each host is assembled from the files in the repo,
including files pulled in with a plain `import ./x.nix`:

![Module tree](./modules.svg)

Both diagrams above are the two-host test fixture, rendered by the same code
path you get. The wiki it produces alongside them is published as the
[live demo](./demo.md).

That fixture happens to run headscale and tailscale, and nixdiag knows nothing
about either. Its mesh node is `services.tailscale.enable = true` plus three
comment lines, so the same graph comes out for wireguard, nebula or a mesh of
your own. Roles are any word you like, and only the scope vocabulary
(`public`, `mesh`, `lan`) is fixed.

## What it reads

From eval, via a single generic projection: enabled services and programs with
their defining files, firewall ports, users, platform, stateVersion, package
count. From the repo source: `#:` annotations, `/** */` doc comments, and
`flake.lock` — a plain file read, so the input graph costs no eval and no
build.

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

- [Quickstart](./quickstart.md): first render in two commands.
- [Annotations](./annotations.md): the `#:` grammar, frozen since 2026-08-26.
- [Build and serve](./build.md): `mkDocs` as a pure derivation, nginx module.
- [CLI](./cli.md): `facts`, `render`, `gen`, `check`.
