# nixdiag

Rust CLI + Nix flake that **statically generates infrastructure docs from any Nix flake**:
data-flow topology diagrams (d2 → SVG), module-tree diagrams, and an mdBook wiki, derived
from `nixosConfigurations` / `darwinConfigurations`. Declared state only — no runtime
metrics (Beszel/Grafana own live state).

Full design spec: https://claude.ai/code/artifact/e1c211d1-a06b-44fb-8cf8-76914e77e2c8

## Origin / reference implementation

This is an extraction of `~/os/automations/{nixdiag,gen-topology,gen-diagram,gen-wiki}.py`
(~500 LOC Python) into a reusable, distributable tool. Port those scripts **to parity
first**, keeping their heuristics: nginx upstream resolution (`proxyPass` + `set $var`),
headscale address book (base_domain + policy.json hosts), `definitionsWithLocations`
service→file mapping, store-path→repo-relative via the `-source/` marker, `AUTO` marker
on generated files.

## Architecture (decided — don't relitigate)

- **Extract / render split.** Nix projection expressions live as `.nix` files in
  `nix/projections/` — the single source of truth. The Rust binary never evaluates Nix
  logic itself; it consumes a schema-versioned `facts.json` (`"schema": 1`). Bump the
  schema on any breaking change to the model.
- **Two modes, one binary:**
  - **A — zero-touch CLI**: `nixdiag gen --flake . [--out docs] [HOST…]`. Projections
    embedded via `include_str!`; discovers hosts, runs **one merged projection eval per
    host** (not several like the Python), parallel across hosts (rayon); needs `nix` +
    `d2` at runtime (wrapped onto PATH by the Nix package).
  - **B — pure derivation**: `nixdiag.lib.mkDocs { pkgs; flake = self; extraPages = …; }`.
    The consumer flake evals its own configs at eval time, `builtins.toJSON` → facts.json,
    `runCommand` runs `nixdiag render` + d2 in the sandbox. `nix build .#docs` is pure and
    cached. This mode is why the split exists — it replaces CI commit-back of docs.
- **CLI subcommands**: `facts` (eval → facts.json on stdout), `render` (facts.json → out
  dir, no nix needed), `gen` (= facts + render), `check` (regenerate to tmp, diff against
  committed docs, exit 1 on drift — the CI gate).
- **Doc comments**: leading RFC 145 `/** … */` block in a module file = file-level doc
  (Markdown body). Parse with the `rnix` crate, not regex. Attachment is automatic: file
  defining a service → that service's wiki entry; a host's entry module → host description.
  No annotation grammar in v1. Hand-written pages come in via `mkDocs.extraPages` /
  `--extra-page` (generalizes the old `write_once` index.md behavior).
- **NixOS module** (`nix/module.nix`): `services.nixdiag.serve` points an nginx vhost
  root at the docs derivation — no daemon, no timer; docs ship atomically with each
  deploy. Optional `services.nixdiag.timer` (oneshot + OnCalendar, pulls a ref, runs
  `gen`) as fallback. Caveat: projections must never force the docs derivation (they read
  vhost names/listen/proxyPass, never `root`) or eval recurses — keep it that way.

## Planned layout

```
flake.nix               # packages.default, apps, overlays.default, nixosModules.default,
                        # lib.{mkFacts,mkDocs}, templates.default, checks, devShells
Cargo.toml              # clap (derive), serde, serde_json, rayon, rnix
src/main.rs             # subcommand dispatch
src/facts.rs            # serde model — THE contract
src/eval.rs             # mode A: spawn `nix eval --json --apply`, parallel
src/doccomment.rs       # rnix scan for leading /** */
src/topology.rs         # port of gen-topology.py
src/modules.rs          # port of gen-diagram.py (module tree)
src/wiki.rs             # port of gen-wiki.py (mdBook src + SUMMARY merge)
src/d2.rs               # spawn `d2 --layout elk`
nix/projections/*.nix   # shared via include_str! AND exported in flake lib
nix/lib.nix             # mkFacts / mkDocs
nix/module.nix          # serve / timer
templates/default/      # `nix flake init -t` consumer scaffold
tests/fixture/          # mini flake with 2 fake hosts → golden-file tests
```

## Rules

- Package with `rustPlatform.buildRustPackage`; the in-repo flake uses
  `cargoLock.lockFile` (no hash churn during dev), the eventual `pkgs/by-name`
  PR uses `cargoHash` against a release tarball. Wrap `d2` and `nix` onto PATH
  with `makeWrapper` (`--suffix`, so the user's own binaries win).
- The renderer **refuses to overwrite an existing file that lacks the AUTO marker**
  (`<!-- Auto-generated … -->` / `# Auto-generated …`). Safer than the Python.
- Golden tests: render the fixture flake, snapshot d2 + Markdown, compare in
  `nix flake check`. Update snapshots deliberately, never automatically.
- Plain `nix eval` errors on one host degrade gracefully (warn + skip), like the Python.
- Darwin hosts must eval from Linux (eval only, no builds) — CI is a Linux Gitea runner.

## Decisions made during the port

- Projections wrap null-able option values (`str'` helper): NixOS options can be
  *defined* with a null default, which `or` does not cover.
- Canonical host order (nixos sorted, then darwin sorted) is enforced by
  `Facts::normalize()` in the renderer, so mode A (discovery order) and mode B
  (alphabetical `builtins.toJSON`) produce identical documents.
- `--extra-page TITLE=FILE` copies a hand-written page in; `--extra-link
  TITLE=NAME.md` only adds the SUMMARY entry (for pages another tool writes into
  wiki/src, e.g. termux artifacts). mkDocs mirrors both as `extraPages` /
  `extraLinks` attrsets.
- `check` compares only deterministic outputs (`.d2` + auto `.md`); SVG varies
  with the d2 version and write-once files (index.md, book.toml) are user-owned.
- Mode A defaults can be declared in the documented flake as a `nixdiag = {
  out, title, extraPages, extraLinks }` output (evaluated silently, absent is
  fine); CLI flags override, `out` resolves relative to the flake root.
- vhosts are projected regardless of `nginx.enable` — parity with the Python
  (the committed os docs list the module's default `localhost` vhost too).
  Candidate cleanup for later, post-parity.

## To do

- [x] Scaffold: `flake.nix`, `Cargo.toml`, binary via `buildRustPackage`,
      devShell, CI workflow (`.gitea/workflows/check.yaml`).
- [x] `facts.rs` model + `nix/projections/{nixos,darwin}.nix` (one merged
      projection per host kind).
- [x] `eval.rs` (mode A) + `nixdiag facts`.
- [x] `topology.rs`, `modules.rs`, `wiki.rs`, `d2.rs` + `nixdiag render` / `gen`.
      Verified byte-for-byte against the committed os docs (modulo marker lines).
- [x] `nixdiag check` drift gate.
- [x] `nix/lib.nix` `mkFacts`/`mkDocs` (mode B) + `tests/fixture/` + golden
      tests in `checks` (`nix build .#fixture-docs` refreshes; copy with
      `cp --no-preserve=mode`, store files are read-only).
- [x] `doccomment.rs` (`/** */` via rnix) — host entry-module doc under the
      host heading, service file docs as sections on the services page.
- [x] `nix/module.nix` serve (+ optional timer), `templates/default/`.
- [ ] Migrate `~/os`: add input, `packages.docs` with `extraPages`/`extraLinks`
      for termux, delete the four Python files, replace `diagrams.yaml`
      commit-back with `nix build .#docs` + rsync, re-point the deploy workflow
      trigger, optionally enable serve on nas.
- [x] README with the zero-touch one-liner + prior-art note.
- [ ] Later: nixpkgs PR; roadmap projections one at a time — flake inputs graph
      (`nix flake metadata --json`, follows edges), sops-nix secrets map, systemd timer
      calendar, disko/zfs storage layout, headscale ACL matrix, users×hosts access
      matrix, firewall-vs-listeners port map.
