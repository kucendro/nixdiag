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

## Layout

The module tree mirrors the pipeline: **eval a flake -> facts; read the repo's
source for intent; render documents.** Split by that boundary, not by file size
— `source/` reads *text* (comments included, which eval cannot see), `render/`
consumes the evaluated facts plus what `source/` found.

```
src/
  main.rs                 mod decls + dispatch, nothing else
  facts.rs                serde model — THE contract
  eval.rs                 mode A: spawn `nix eval --json --apply`, parallel
  util.rs
  cli/
    mod.rs                clap types (Cli, Cmd, FlakeArgs, RenderArgs) + run()
    options.rs            flake `nixdiag` output + flags -> RenderOpts/D2Style
    commands.rs           facts / render / gen / check
  source/                 static analysis of the documented repo's own .nix files
    repo.rs               store path -> repo-relative (the `-source/` marker)
    doccomment.rs         leading RFC 145 /** */ via rnix
    imports.rs            host entry modules + the import graph
    flakelock.rs          flake.lock -> input graph + duplicate detection
    annotations/          the `#:` engine — biggest subsystem, one concern per file
      mod.rs              the grammar doc comment + the public façade
      grammar.rs          GRAMMAR/VERSION, editions, deprecation table
      stmt.rs             the statement grammar (parse only)
      scan.rs             the rnix pass: find statements, record where they sit
      attach.rs           syntactic position + facts + imports -> which nodes
      resolve.rs          raws -> Model (two passes; address book before edges)
      model.rs            Scope, Expose, NodeInfo, Endpoint, Edge, Model
      diag.rs             Sev + Diag (file:line on everything)
  render/
    mod.rs                RenderOpts, render_all
    out.rs                Out/WKind — the AUTO-marker guarded writer
    d2.rs                 palette, vars block, spawn `d2 --layout elk`
    topology.rs           the data-flow diagram
    modules.rs            the module-tree diagram
    inputs.rs             the flake input graph
    wiki/                 mdBook source, one module per generated page
      mod.rs              WikiOpts, page order, shared host->services helper
      book.rs             book.toml, SUMMARY, index, extra pages
      architecture.rs / hosts.rs / services.rs / endpoints.rs / inputs.rs
nix/projections/core.nix  shared via include_str! AND exported in flake lib
nix/lib.nix               mkFacts / mkDocs
nix/module.nix            serve / timer
templates/default/        `nix flake init -t` consumer scaffold
tests/fixture/            mini flake with 2 fake hosts
tests/reference/          snapshots compared in `nix flake check`
site/                     hand-written mdBook, published to GitHub Pages
.github/workflows/        pages deploy only; CI proper lives in .gitea/
```

Rules that keep it that way:

- `main.rs` stays a dispatcher. Anything it would grow goes in `cli/`.
- Nothing in `render/` parses Nix source, and nothing in `source/` writes files.
- `annotations/mod.rs` is a façade: submodules are private, the crate sees only
  the `pub use` list. Cross-module items are `pub(super)`, never `pub`.
- Tests live beside the code they exercise, so `cargo test` output reads as a
  table of contents (`source::annotations::scan::tests::…`).

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
- User-facing docs live in `site/` (mdBook, `nix build .#site`, `checks.site`),
  published to <https://kucendro.github.io/nixdiag> by
  `.github/workflows/pages.yml` as a Pages artifact — no gh-pages branch, which
  the Gitea push mirror would prune. README stays a landing page: hero, one
  example, links. The site nests `packages.demo-docs` (the fixture's own wiki)
  at `/demo` and reuses `assets/*.svg`, so fixture, snapshots, README images and
  demo cannot drift apart.
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

## Versioning and deprecation (policy, decided 2026-08-26)

Three surfaces version independently; they fail differently and must not be
conflated.

1. **Facts schema** (projection ↔ binary): `SCHEMA` / `schema = 2`, checked in
   `render/mod.rs`, fatal on mismatch. Both halves normally ship from one flake, so
   skew only happens when a `lib` from one rev meets a binary from another (the
   nixpkgs binary against a pinned flake `lib`, say). Keep it fatal; the error
   should name both versions, not just the numbers.
2. **Annotation grammar** (user repos ↔ binary): the only surface that outlives
   nixdiag versions, because the annotations live in *other people's files*.
   Edition model, below.
3. **Package API**: CLI flags, `mkDocs` args, module options — and the rendered
   output itself, which is an API too (see below).

**Grammar = editions** (Cargo's model, lighter):

- A `GRAMMAR: u32` constant in the binary, printed by `nixdiag --version` so
  bug reports carry it.
- Optional declaration by the consumer: flake `nixdiag.grammar = 1;`, mkDocs
  `grammar`, `--grammar`, same override order as every other setting. Unset
  means "whatever this binary implements", so zero-config stays zero-config.
- Old binary, newer declared grammar → hard error naming both numbers and
  pointing at the input bump. Better than guessing at an unknown statement.
- New binary, older declared grammar → compatibility mode: old spellings work,
  deprecated ones warn, nothing is removed *inside* an edition.
- Removals happen only at an edition bump, never in a patch or minor.

**Deprecation lifecycle** per statement:

1. Introduce the replacement in a minor release; the old spelling keeps
   working and the renderer warns `file:line: #: <old> deprecated since 0.5,
   use #: <new>` (every annotation already carries file and line).
2. `nixdiag check --deny deprecated` for consumers who want CI red at once.
3. Remove at the next edition, with an error naming the replacement and the
   migration command. No silent behaviour changes, same reason malformed lines
   are fatal.

**Migration**: `nixdiag migrate --to N` rewrites the comment lines in place
through the existing rnix scan; one statement per line is what makes this
mechanical, and the user reviews a diff. This is the pay-off for the format's
constraints and the thing that makes an edition bump cheap enough to actually
perform.

**Rendered output is an API for mode A.** Consumers commit `docs/` and gate CI
with `check`, so a purely cosmetic renderer change (reworded label, column
order, d2 layout) turns every consumer's CI red until they re-run `gen`. So:
output changes are a minor bump at minimum plus a changelog entry, and `check`
should hint "if you just upgraded nixdiag, run `nixdiag gen`". Mode B consumers
are structurally immune (nothing committed, input pinned) — another reason
`mkDocs` is the recommended path, especially for nixpkgs users who pin nothing.

**Do not stamp versions into the `Auto-generated` markers**: it would churn
every reference snapshot here and every consumer's committed docs on each
upgrade. If provenance in the output is wanted, it goes in a small
`.nixdiag.json` manifest in the output dir (`{version, grammar}`), never in the
pages.

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
- [x] Docs site: `site/` mdBook (overview, quickstart, annotations, build and
      serve, CLI), `packages.{site,demo-docs}` + `checks.site`, Pages workflow,
      README trimmed to a landing page.
- [x] v2 annotation engine (spec in "v2 direction" above): `#:` parser (rnix,
      render-side, now `src/source/annotations/`), generic topology model, single
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
- [x] Versioning mechanics (policy above): `GRAMMAR` + `VERSION` from one
      `def_grammar!` macro in `annotations/grammar.rs` (the parser owns the constant it
      implements), `resolve_edition` with both skew errors, `--grammar` /
      `nixdiag.grammar` / mkDocs `grammar`, deprecation warnings +
      `--deny deprecated`, schema error widened to name both nixdiag versions.
      Implementation notes:
      - Deprecation is a **verb-level rewrite before `parse_stmt`**
        (`canonicalize`), not a change to the grammar match itself — so
        `parse_stmt` and its tests were untouched. `canonicalize` takes the
        table as a parameter, which is what lets the tests drive the mechanism
        with a synthetic entry while the real `DEPRECATIONS` is empty. Roles
        are free-form and sit in verb position, so a role rename is covered by
        the same table.
      - `Diag` grew `Sev { Error, Deprecated }`; the variant names *are* the
        `--deny` vocabulary, so a future warning category is one variant plus
        one accepted flag value. `render_all` prints `error:`/`warning:` and
        counts only fatals.
      - `--deny` lives in `RenderArgs`, so `render`/`gen`/`check` all take it
        (the policy text only named `check`) — mode B needs it on `render` to
        be a usable CI gate at all.
      - `cmd_render` uses `FlakeConfig::default()`, so mode B never sees the
        flake's `nixdiag` attr: `mkDocs` must pass `grammar`/`deny` explicitly,
        same as `title`/`theme`/`domains`.
- [x] `CHANGELOG.md` (Keep a Changelog; one `## Unreleased` section — nothing
      is tagged yet, version stays 0.1.0) and the `check` upgrade hint.
- [ ] `nixdiag migrate --to N`: in-place comment rewriter over the rnix scan,
      reviewed as a diff. Only needed at the first edition bump, but nothing in
      the frozen grammar may break its feasibility (one statement per line, no
      continuations).
- [ ] Optional `.nixdiag.json` output manifest (`{version, grammar}`) if
      provenance is ever wanted; markers stay version-free.
- [ ] v2.x: reference plucks (`cfg.*`) + options-tree validation in `check`.
- [x] Atlas triage (`visualizations.md`, 21 plates). Held against the bar, only
      six survive; the disqualifier is nearly always that the plate needs a
      build to have happened (3.x), a live store (5.x, 2.2), a clock, or debug
      output that is not a contract (1.x). The survivors are exactly the two
      families the sketch's own triage called best-ratio: flake-lock arithmetic
      and closure size. **Liveness/status is permanently out** — Beszel and
      Prometheus/blackbox already own it, and mixing world-at-time-T into a
      derivation would kill mode B's reproducibility.
- [x] Flake inputs (plates 6.1/6.2/6.3): `source/flakelock.rs` +
      `render/inputs.rs` + `render/wiki/inputs.rs`. Decisions:
      - `flake.lock` is a plain file read — no eval, no realisation, no clock —
        so this needs zero new plumbing (`render` already takes `--repo`) and
        behaves identically in both modes. Default on. `lastModified` is a
        fixed integer *in the lock*, which is what keeps dates deterministic;
        "overdue" would need a clock and is therefore not rendered.
      - Two duplicate signals, deliberately separated because they need
        different reactions: a **diamond** (one repo at several revs — a
        correctness risk, reported with the `follows` that fixes it) versus
        **redundancy** (one rev under several node names — a wasted fetch).
      - Identity folds case on forge `owner`/`repo`. Without it `nixos/nixpkgs`
        beside `NixOS/nixpkgs` is missed — which is exactly the diamond the
        ~/os lock actually had (sops-nix pulling its own nixpkgs).
      - A `follows` is only suggested when the root has an input to point at,
        so no advice is invented for a repo the root never pulls.
      - Only diamond nodes carry their rev in the d2 label; a rev on every box
        is noise. Reuses existing `PALETTE` names rather than adding any, so
        `topology.d2`/`modules.d2` stayed byte-identical — `vars_block` emits
        the whole palette into every diagram, so a new entry would churn every
        snapshot here and in every consumer's committed docs.
      - The `![…](./inputs.svg)` line is emitted whether or not the SVG was
        rendered: `check` runs with `--no-svg` and the Markdown must not differ.
      - `tests/fixture/flake.lock` is hand-written and exercises every branch
        (diamond with case difference, redundancy, a `follows`). Safe because
        `tests/fixture/flake.nix` is a plain attrset that is never evaluated as
        a flake, so nothing will ever rewrite the lock.
- [x] Closures (plates 4.1/4.2/4.4): `nix/closures.nix` + `src/closures.rs` +
      `render/wiki/closures.rs`, opt-in via `mkDocs { closures = true; }`.
      Decisions:
      - **Mode B only.** Nar sizes exist only for *realised* paths (Nix 2.34.8:
        `nix path-info` "does not build or substitute"; exportReferencesGraph
        errors `cannot export references of path '%s' because it is not in the
        input closure`). Only a derivation can depend on builds purely, so
        `gen --closures` / `check --closures` exist solely to refuse with a
        pointer at `mkDocs`.
      - **Separate `closures.json`, own schema** — not more fields on facts.
        The provenance differs (realisation vs evaluation) and `mkFacts` is
        pure eval, so it structurally cannot fill them. `facts.json` stays
        schema 2 and no consumer breaks.
      - `__structuredAttrs = true` is **load-bearing**: the classic
        exportReferencesGraph text format is `path / empty deriver / nrefs /
        refs` with *no sizes*. Structured attrs replace the path list with
        PathInfo objects carrying `narSize`. Depend only on `path`/`narSize`
        and tolerate extra keys. Under structuredAttrs `$out` is not set the
        usual way — take `out=''${outputs[out]}`, as closureInfo does.
      - One derivation per host, not one multi-key derivation:
        exportReferencesGraph keys land at the *top level* of `.attrs.json`,
        where a host named `outputs` or `builder` would collide.
      - Totals/counts are always derived from `paths`, never stored, so they
        cannot disagree with the list they summarise.
      - **The serve cycle, and why `mkDocs` breaks it itself.**
        `services.nixdiag.serve` roots an nginx vhost at a docs derivation, so
        a serving host's closure *contains* the docs; measuring it would give
        `docs -> toplevel -> docs`, which Nix surfaces only as infinite
        recursion. Verified with `throw` tripwires rather than by building the
        cycle (safer, and it says who forced what): reading
        `serve.enable` does **not** force `serve.docs`, while
        `system.build.toplevel` **does**. That asymmetry is what makes
        detection possible, so `closures = true` filters serving hosts out and
        `lib.warn`s naming them. `closures = [ … ]` is taken at the user's
        word and never filtered — needed because serving one build while
        measuring another is legitimate and has no cycle, and nixdiag cannot
        tell the two apart without forcing `serve.docs`.
      - **Never print a store path into generated output.** Nix scans build
        outputs for store-path strings and records each as a real reference:
        measured, a 367-byte markdown table listing five paths had a 36 MiB
        closure. Printing a system closure would make the docs derivation
        retain every path it describes, and `serve` would pull that into the
        serving host's system. So the page lists `linux-6.12.9`, not
        `/nix/store/<hash>-linux-6.12.9` (`util::store_name`), and
        `checks.closures` greps the whole output tree for
        `/nix/store/<32>-` so it cannot regress. Full paths *are* kept in
        `closures.json`, because the shared/unique set analysis needs real
        identity — two hosts can hold different builds of the same name — and
        that file is a transient build input, not a GC root.
      - Testing an eval cycle safely: never bound it with `timeout` alone,
        which does not cap memory. Prefer `throw` tripwires; if a real cycle
        must be evaluated, use `--option max-call-depth` plus `ulimit -v` or a
        systemd scope with `MemoryMax`.
      - Tests build no NixOS system: `checks.closures` renders
        `tests/fixture/closures.json` (hand-written, deliberate overlap) and
        diffs `tests/reference/closures.md`; `checks.closures-plumbing` runs
        the real derivation over `pkgs.hello` and asserts shape *and* sort
        order. The end-to-end path over real hosts is verified by hand.
      - `wiki::generate` grew past clippy's argument limit, so the data inputs
        (facts, repo, docs, model, lock, closures) are bundled into
        `WikiData`. Add future inputs there, not to the signature.
      - **It measures, it does not build a fleet** (settled 2026-09-01 while
        checking whether a `closures` build could double as a binary-cache
        warm). It can't, on its own: the same no-store-paths rule that stops
        `serve` retaining a fleet means the docs hold *no* references to the
        systems they measure, so everything realised is unrooted and the next
        GC takes it. The working arrangement inverts the order — your own
        pipeline builds and roots the systems (`--out-link` is the GC root),
        the docs build runs after and finds everything realised, and a cache
        on that machine serves the systems as a side effect. ~/os already had
        exactly this in `.gitea/workflows/update.yaml`. Documented on the
        `mkDocs` arg, in `site/src/build.md`, and here; no code needed.
      - `preferLocalBuild` on the per-host derivation is **correct, not a
        bug** — checked and nearly "fixed" the wrong way. It pins the jq to
        the invoking machine, which is what you want when the docs build runs
        where the systems are; removing it would let Nix ship a whole system
        closure *to* a remote builder, since Nix does not schedule on input
        locality. The real failure mode is invoking the build from a machine
        that is not the one holding the systems, and that is an instruction,
        not a flag.
      - Unmeasured NixOS hosts get a row with `—` rather than being dropped:
        `closures` takes an opt-in list, and a silent omission reads as "this
        is the whole fleet". On the hosts page the row says `not measured`,
        which is distinguishable from the feature being off (no row at all).
        `summary_rows` is split out of `page_closures` purely so that case is
        unit-testable without constructing a whole `Facts`.
- [ ] Later: nixpkgs PR; extra diagrams/pages one at a time, each held to the
      v2 bar — derivable from stable generic surfaces (e.g. flake inputs graph
      via `nix flake metadata --json`, systemd timer calendar from units) or
      driven by annotations/references, never by hardcoded option-shape
      walkers (which rules out the old sops-nix/disko/ACL-matrix ideas unless
      they pass that test).
