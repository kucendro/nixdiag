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
  No annotation grammar in v1 (v2 adds one — see "v2 direction" below). Hand-written
  pages come in via `mkDocs.extraPages` / `--extra-page` (generalizes the old
  `write_once` index.md behavior).
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
tests/fixture/          # mini flake with 2 fake hosts → reference-file tests
```

## Rules

- Package with `rustPlatform.buildRustPackage`; the in-repo flake uses
  `cargoLock.lockFile` (no hash churn during dev), the eventual `pkgs/by-name`
  PR uses `cargoHash` against a release tarball. Wrap `d2` and `nix` onto PATH
  with `makeWrapper` (`--suffix`, so the user's own binaries win).
- The renderer **refuses to overwrite an existing file that lacks the AUTO marker**
  (`<!-- Auto-generated … -->` / `# Auto-generated …`). Safer than the Python.
- Reference tests: render the fixture flake, snapshot d2 + Markdown, compare in
  `nix flake check`. Update snapshots deliberately, never automatically.
  The README's example diagrams are rendered from the reference files — whenever they
  change, refresh with
  `d2 --layout elk --theme 200 tests/reference/<x>.d2 assets/<x>.svg`
  (`--theme 200` because the default palette is dark).
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
- mkDocs grew `indexPage` / `bookToml` (user-owned files that `write_once`
  would otherwise reseed with defaults in the fresh sandbox) and `extraAssets`
  (files/dirs copied into wiki/src before mdbook — e.g. images referenced by
  extra pages). Gotcha found in the os migration: mdbook resolves a relative
  `--dest-dir` against the **cwd**, not the book root — always pass
  `$out/wiki/book` absolute.

## v2 direction: annotations, not built-in knowledge (decided 2026-08-26)

Schema 1 hardcodes one stack (tailscale/headscale/nginx/prometheus/beszel) into
projections, schema, and renderer. Right for parity, wrong permanently — and not
only because it is *our* bias: any service-specific extraction encodes *mechanism*
(how a module's options are shaped this month, which nixpkgs-unstable reshapes
constantly), while docs should record *intent* ("this is my proxy; it fronts
grafana"), which survives every upstream refactor. v2 therefore has **no adapter
layer at all** (an earlier draft kept the heuristics as opt-in adapters —
rejected: each adapter is a permanent promise to track nixpkgs, nix-topology's
known pain, and its rot is silent). All topology semantics come from the user's
own files. The visual language (d2 styles, scopes, wiki layout) does not change,
and rendering stays text-only: no icon/image assets, ever (contrast nix-topology).

- **What v2 still reads from eval** (`nix/projections/core.nix`, schema 2):
  enabled services/programs + defining files (module-system introspection),
  firewall ports, users, platform, stateVersion, pkgCount. Quasi-frozen NixOS
  API — acceptable coupling. All schema-1 service-specific fields and the Rust
  topology heuristics die.
- **Topology annotations**: comment lines in the user's own module files, sigil
  `#:` (two chars — written often, must be cheap); `# nixdiag:` accepted as the
  self-documenting long alias; the same lines are recognized inside leading
  `/** */` doc comments. Comments are invisible to eval, so annotations are
  parsed render-side with rnix (the renderer owns the repo source in both modes).
  **Attachment**: a line directly above a `services.<x>` / `programs.<x>` binding
  attaches to that service; in a file-leading doc comment it attaches to whatever
  that file defines (reverse `definitionsWithLocations`); host entry modules take
  host-level annotations.
- **Grammar** (one statement per line; a malformed line is a reported error,
  never silently ignored; `role` is the implicit verb, `->`/`<-` are edges):
  - `#: mesh-control` — role; known roles (mesh-control, mesh-node, proxy,
    monitor, agent, dns, storage, gateway) map to d2 classes + placement;
    unknown role names still render with defaults, so diagrams stay
    user-programmable without touching nixdiag.
  - `#: expose <port>[/udp] [public|mesh|lan] [name=<fqdn>]`
  - `#: -> <host[/service] | fqdn> [label] [name=<fqdn>[:port]]` (and `<-`) —
    any *enabled* service is a valid edge target for free (the generic core
    knows them all), so references resolve against real state, not strings.
    `name=` marks the fronted endpoint the annotated node serves for that
    target: an Endpoints page row (scope from the node/host `#: scope`, port
    user-stated, `—` without one); the diagram is unaffected.
  - `#: name <fqdn>` — address-book entry resolving that fqdn to this node
  - `#: scope public|mesh|lan`
  - `#: unit <[host/]name>` — declares/targets a node; its contiguous block
    attaches to it. In the file-leading doc comment it sets the file's
    DEFAULT attachment: file-level annotations anywhere in that file attach
    to it (per-binding attachment still wins), so a plain-data file (an
    upstream table pulled in via `import ./x.nix`) carries annotations next
    to its entries. The `host/` pin disambiguates files that several hosts'
    import graphs reach (the os case: proxy and blackbox monitoring import
    the same endpoints table — unpinned, everything duplicated onto nas).
    The import graph follows both `imports = [ ... ]` lists and plain
    `import ./path` expressions.
  - Any fqdn position accepts `<sub>@<key>` (bare `@<key>` is the domain
    itself): the suffix comes from a user-declared domain map (`--domain
    KEY=DOMAIN`, flake `nixdiag.domains`, mkDocs `domains`; CLI overrides
    flake), unknown key is a hard render error. Added at the os dogfood: the
    proxied-vhost endpoints needed fqdns that public repo source must not
    disclose, and comments cannot interpolate Nix values — the map injects
    the private suffix at render time.
  - Grammar FROZEN 2026-08-26 after the ~/os dogfood. Additive evolution only
    (new statements/optional tokens are fine); renames or meaning changes are
    a deliberate break with a version bump. Accepted semantics: sub-services
    fold into the parent unit key unless split via `unit`; proxied vhosts get
    Endpoints rows only through opt-in `name=` on their edges.
- **Zero annotations**: wiki, modules diagram, hosts/services/ports pages are
  unaffected; the topology renders hosts + firewall ports with no edges, plus a
  stderr hint pointing at the annotation docs.
- **Reference plucks (v2.x, additive — not in the first cut)**: `#: expose
  cfg.port public` resolves `cfg.*` against the annotated service's *evaluated*
  config — Nix supplies the value, and the option-shape coupling lives in the
  user's repo, owned by them. Every referenced path is validated against the
  introspectable `options` tree at generation time; a vanished path fails
  `check` loudly with a fuzzy-matched suggestion. nixpkgs' own rename machinery
  cushions this: `mkRenamedOptionModule` aliases keep old paths *readable*
  through the deprecation window, and `mkRemovedOptionModule` throws naming the
  replacement — drift surfaces as a named error, never silent rot. Eval
  *warnings* are NOT a usable signal here: they fire when a config **sets** a
  deprecated option, not when a reader references one, and they are free text.
- **No LLM in the generation path.** Mode B is a sandboxed derivation and must
  stay pure/offline (this is a feature, not a limitation). Keeping annotations
  current is dev-time work driven by `check` diagnostics; an assistant may help
  *there*, outside the tool.

## To do

- [x] Scaffold: `flake.nix`, `Cargo.toml`, binary via `buildRustPackage`,
      devShell, CI workflow (`.gitea/workflows/check.yaml`).
- [x] `facts.rs` model + `nix/projections/{nixos,darwin}.nix` (one merged
      projection per host kind).
- [x] `eval.rs` (mode A) + `nixdiag facts`.
- [x] `topology.rs`, `modules.rs`, `wiki.rs`, `d2.rs` + `nixdiag render` / `gen`.
      Verified byte-for-byte against the committed os docs (modulo marker lines).
- [x] `nixdiag check` drift gate.
- [x] `nix/lib.nix` `mkFacts`/`mkDocs` (mode B) + `tests/fixture/` + reference
      tests in `checks` (`nix build .#fixture-docs` refreshes; copy with
      `cp --no-preserve=mode`, store files are read-only).
- [x] `doccomment.rs` (`/** */` via rnix) — host entry-module doc under the
      host heading, service file docs as sections on the services page.
- [x] `nix/module.nix` serve (+ optional timer), `templates/default/`.
- [x] Migrate `~/os`: input added, `packages.docs` + `checks.docs`, Python
      generators deleted, committed `docs/` dropped entirely (repo going
      public), wiki served tailnet-only on edge via `services.nixdiag.serve`.
- [x] README with the zero-touch one-liner + prior-art note.
- [x] v2 annotation engine (spec in "v2 direction" above): `#:` parser (rnix,
      render-side, `src/annotations.rs`), generic topology model, single
      generic `core.nix` projection, schema 2, role→d2 class map; schema-1
      service fields and the Rust heuristics deleted. First-cut decisions
      (revisit at the dogfood, grammar not frozen yet):
      - Attachment: an annotation binds to the nearest enclosing/following
        `services.<x>`/`programs.<x>` attrpath; it must be on its own line
        (trailing comments error). Sub-service enables (`services.x.hub`) are
        invisible to the generic projection, so attachment falls back to the
        hosts whose import graph reaches the file. File-level lines: host
        entry module → host; else the units the file defines; else error.
      - `#: unit <name>` (added at the os dogfood) declares a node the
        projection can't see — an OCI container, a raw/generated systemd
        unit; placed on the hosts whose import graph reaches the file. A
        contiguous run of `#:` lines is one block, and a `unit` declaration
        re-attaches its whole block (also usable to split a sub-service from
        its parent unit, e.g. beszel agent vs hub). The alternative — parser
        recognition of `systemd.services.*` / `oci-containers` attrpaths —
        was rejected as tool-side bias: the parser knows only
        `services.`/`programs.` bindings, everything else is declared.
      - Malformed lines and dangling edge targets are hard render errors —
        mode B's docs build fails, which is the intended CI gate.
      - An expose with no scope (own, service, or host `#: scope`) makes no
        public/lan claim: no cloud edge, `—` on the Endpoints page.
      - Builtin edge targets `internet`/`lan`; a bare service name resolves
        iff exactly one host enables it; fqdns resolve via `#: name` entries.
      - Roles mesh-control/proxy/monitor/dns/storage/gateway → `infra` class,
        everything else (incl. unknown) → `app`; label = role with `-`→space.
      - Colors all live in a d2 `vars` block per diagram; built-in dark
        (default) and light sets, user-tunable via `--theme dark|light`,
        `--background` (default transparent) and repeatable
        `--color name=#hex` (mkDocs: `theme`/`background`/`colors`, also
        valid in the flake `nixdiag` attr). Semantic defaults: public red,
        lan green, mesh blue. d2 0.8.1 has no theme-code color refs and
        `--dark-theme` cannot switch explicit styles (d2lang/d2#831), so one
        build = one theme; dark also passes `--theme 200` to d2 for
        label/text colors. A d2 compile failure is now a hard render error
        (was a silent eprintln).
      - Scope keyword is `mesh`, not `tailnet`: the grammar stays
        stack-agnostic (any overlay), matching the mesh-control/mesh-node
        roles.
- [x] v2 dogfood: ~/os modules annotated, rendered diagrams verified against
      the schema-1 heuristic output, grammar frozen 2026-08-26. Grew `unit`,
      edge `name=` and the `@key` domain map (see Grammar above).
- [ ] v2.x: reference plucks (`cfg.*`) + options-tree validation in `check`.
- [ ] Later: nixpkgs PR; extra diagrams/pages one at a time, each held to the
      v2 bar — derivable from stable generic surfaces (e.g. flake inputs graph
      via `nix flake metadata --json`, systemd timer calendar from units) or
      driven by annotations/references, never by hardcoded option-shape
      walkers (which rules out the old sops-nix/disko/ACL-matrix ideas unless
      they pass that test).
