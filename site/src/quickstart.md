# Quickstart

## Render an existing flake

```sh
cd ~/my-nix-config
nix run github:kucendro/nixdiag
```

Prints the store path of the docs. Open `wiki/book/index.html` in it.

## Start a new flake

```sh
nix flake init -t github:kucendro/nixdiag
nix build .#docs
```

## First override

```nix
{
  nixdiag.units.nginx.role = "gateway";
  nixdiag.units.exporter.connections = [ { to = "grafana"; label = "metrics"; } ];
}
```

More in [Topology](./topology.md).

## Keep it honest in CI

```nix
checks.x86_64-linux.docs = self.packages.x86_64-linux.docs;
```

An unresolved connection fails `nix flake check`.
