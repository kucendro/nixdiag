# v2 tests

Fixture made up. Snapshot made up. Adapter test made up. Stop.

## Idea

Adapter claims. VMs run. Compare both ways.
Claim: adapter facts at that head. Truth: running VMs.
No expected output. Nobody writes it.

## Adapter

Adapter says where it runs and what it needs. `tests`:

- `vm`: premade VMs it runs on.
- `minimum`: what service needs to boot.
  Never an option adapter reads, so no path nixpkgs can rename.
  Minimum path hits a read path: eval fails.
- `probe`: optional. Values by read name, not path.
  Harness writes each at first candidate that exists at the head. None exists: eval fails.
  `<name>` reads nest by name.

```nix
tests = {
  vm = [ "mon" ];
  minimum.services.grafana.settings.security.secret_key = "test";
};

tests = {
  vm = [ "web" ];
  probe.proxyPass."grafana.test"."/" = "http://mon:3000";
};
```

No `tests.vm`, or unknown VM: eval fails.

## VMs

Premade base: `tests/vms/<name>.nix`. Boot, network, stateVersions. Knows no adapter.

- Harness builds a node per VM some adapter names.
- Node: base, plus per adapter `enable` through its enable read, `minimum`, probes.
- All nodes in one test. VMs reach each other. Topology is real, across machines.
- Runs per stateVersion in base's list.
- Probe URL names a VM with no node: eval fails.
- Host arch only. Test framework pairs guest with host.

## Checks

From each VM's own `nixdiag.facts`. Nix writes test lines. No JSON.

- claimed port: listens.
- unit listens on port: port claimed.
- connection: through its entry, reaches `to`. Nginx: `Host: <name>` on its port.
- scope: other VM reaches exposed port or not, as scope says.
- tailscale: logged in to claimed server.
  `extraUpFlags` only runs with `authKeyFile`, so script makes headscale key first.

Why both ways: claims-only lines miss a dropped claim.
Rename, read null, no claim, no line, green. Listening port with no claim: red.

## Defaults alone, pinned head

| Adapter | Claims |
|---|---|
| grafana | 3000 |
| headscale | 8080; `base_domain` in minimum |
| nginx | 80, default `localhost` vhost |
| tailscale | nothing checkable |

So probes: `proxyPass`, SSL, `--login-server`.

## Heads

- `nix flake check`: pinned head. CI runner needs kvm, `system-features = nixos-test kvm`.
- monitor: VMs per channel head, `nixos-unstable` and current stable. Cached, cheap.
- `master`: eval and audit only. Not cached, VM builds from source.
- VMs replace fixture topology diff.

## Keeps

- Fixture: render snapshots, demo, README assets. Not adapter truth.
- Audit: cheap, early rename signal.
- Closures: VM system measured for demo and README.
  Snapshots keep hand `closures.json`; real sizes move every lock bump.

## Slices

1. `tests` in adapters, premade VMs, harness, pinned head, `nix flake check`.
2. Monitor runs VMs per channel head, drops fixture diff.
3. Closure render from VM system.
