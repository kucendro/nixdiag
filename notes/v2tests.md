# v2 tests

Fixture made up. Snapshot made up. Adapter test made up. Stop.

## Idea

Adapter claims. VM runs. Compare.
Claim: adapter facts at that head. Truth: running VM.
No expected output. Nobody writes it.

## Per adapter

One file: `tests/adapters/<name>.nix`. Missing file fails eval.

- `minimum`: what service needs to boot. Never an option adapter reads.
  So no option path nixpkgs can rename.
- `probe`: optional. Values by read name, not path.
  Harness writes each at first candidate that exists at the head.

`{ minimum.services.grafana.settings.security.secret_key = "test"; }`
`{ probe.up = [ "--login-server=http://127.0.0.1:8080" ]; }`

## Harness

- One VM. Every adapter on. Adapters reach each other.
- Harness sets `enable` through adapter's enable read.
- Checks from the VM's own `nixdiag.facts`. Nix writes test lines. No JSON.
  - claimed port: listens.
  - connection `to` a URL: answers from its host.

## Defaults alone, pinned head

| Adapter | Claims |
|---|---|
| grafana | 3000 |
| headscale | 8080; `base_domain` in minimum |
| nginx | 80, default `localhost` vhost |
| tailscale | nothing checkable |

So probes: `proxyPass`, SSL, `--login-server`.

## Heads

- `nix flake check`: pinned head.
- monitor: `master`, `nixos-unstable`, `nixos-25.05`. VM per head.
- VM replaces fixture topology diff.

## Keeps

- Fixture: render snapshots, demo, README assets. Not adapter truth.
- Audit: cheap, early rename signal.
- Closures: VM is real system. Measure, render real size. No hand numbers.

## Slices

1. Harness, four adapter files, pinned head, `nix flake check`.
2. Monitor runs VM per head, drops fixture diff.
3. Closure render from VM system.
