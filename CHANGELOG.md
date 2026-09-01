# Changelog

Notable changes to nixdiag. Format follows [Keep a Changelog][kac]; versions
follow [semantic versioning][semver].

Three surfaces version independently and fail differently — see the versioning
policy in `CLAUDE.md`:

- **facts schema** (projection ↔ binary) — fatal on mismatch;
- **annotation grammar** (your module files ↔ binary) — editions, removals only
  at an edition bump;
- **package API** — CLI flags, `mkDocs` arguments, module options, *and the
  rendered output itself*.

The rendered output is an API for mode A: consumers commit `docs/` and gate CI
with `nixdiag check`, so even a cosmetic renderer change turns their CI red
until they re-run `nixdiag gen`. Output changes are therefore a minor bump at
minimum, with an entry here. Mode B (`lib.mkDocs`) consumers are structurally
immune — nothing is committed and the input is pinned.

## Unreleased

Facts schema 2. Annotation grammar 1, frozen 2026-08-26.

### Added

- **Closure metrics** (opt-in, `lib.mkDocs { closures = true; }`). A Closures
  wiki page with per-host totals, the largest contributing packages, and a
  fleet analysis of what the hosts share versus what each one costs on its own,
  plus a closure row on every host. Mode B only: nar sizes exist solely for
  *realised* paths, so producing them means taking every host's
  `system.build.toplevel` as a build input — something only a derivation can
  express purely. `nixdiag gen --closures` and `check --closures` therefore
  refuse with a pointer to `mkDocs`. NixOS hosts only, since darwin cannot be
  built from Linux. The data arrives as a separate `closures.json` with its own
  schema, so `facts.json` stays schema 2 and `mkFacts` stays a pure eval.
  `closures` takes `true` (every NixOS host that does not serve nixdiag docs)
  or an explicit list of host names. Serving hosts are skipped under `true`
  because `services.nixdiag.serve` roots an nginx vhost at a docs derivation,
  so measuring such a host would make the docs depend on a system that
  contains them (`docs -> toplevel -> docs`) — which Nix reports only as
  infinite recursion. The skip is announced by a warning naming the hosts. An
  explicit list is never filtered. NixOS hosts that were not measured are
  still listed, with `—` for their numbers, so an opt-in list cannot read as
  though it covered the whole fleet.

  The measurement is not a way to build a fleet: the pages carry no store
  paths, so the docs hold no references to the systems they describe and a
  later garbage collection removes anything your own pipeline did not root.
  Build and root the systems first (`nix build …system.build.toplevel
  --out-link …`) and enable `closures` on a docs build that runs afterwards —
  every path is then already realised and the measurement costs seconds. Run
  that build on the machine holding the systems: the per-host derivation is
  `preferLocalBuild`, so invoking it elsewhere copies every measured closure
  to the invoking machine.
- `checks.closures-self` measures nixdiag's own runtime closure with
  `nix/closures.nix` and fails if a `playwright` path reappears or the total
  passes 600 MiB — a tripwire, not a budget.
- **Fleet closure chart**, `wiki/src/closures.svg`: one stacked horizontal bar
  per host, divided into paths every host carries, paths some carry, and paths
  unique to that host — the picture of what a host costs *beyond* the shared
  base. Unmeasured hosts keep their row and lose their bar, for the same reason
  they keep their `—` in the table. nixdiag emits this SVG itself instead of
  through d2: node-link renderers have no length or area channel, so the
  quantitative pages could not be drawn with one at all, and a second renderer
  dependency would be a standing commitment to track another upstream. Two
  consequences worth knowing: the chart needs no binary on PATH and so ignores
  `--no-svg`, and it is byte-identical run to run, which makes it the first
  picture `nixdiag check` can compare (every d2-produced SVG is excluded,
  because its bytes move with the d2 version). Colors follow `--theme` and take
  overrides under their own names — `chartShared`, `chartPartial`,
  `chartUnique`, `chartInk`, `chartMuted`, `chartTrack` — which are deliberately
  absent from the d2 `vars` palette so existing diagrams stay byte-identical.
- **Closure treemap**, one `wiki/src/closures-<host>.svg` per measured host:
  every package in that host's closure as a rectangle sized by its share of
  the bytes, squarified, coloured with the same bands as the fleet bar — so it
  reads as that bar exploded by package. Tiles are packages rather than store
  paths (a package's several outputs fold into one), while the table beneath
  stays per path. The long tail past the largest two dozen becomes a single
  counted tile rather than a scatter of slivers.
- **Lock timeline**, `wiki/src/inputs-timeline.svg`: every input placed on a
  shared axis by its `lastModified`, oldest first, with the direct inputs of
  this flake coloured apart from the ones reached through another input —
  because a `nix flake update` moves the first and only the responsible input
  moves the second. Rows sort by date, so the first and last of them label the
  ends of the axis and no separate scale is drawn; an input without a date
  keeps its row and gets no tick. The page states the span between the oldest
  and newest input in days. `lastModified` is a fixed integer in the lock, not
  a clock read, so this is the *spread* of the supply chain and makes no claim
  about today — an "overdue" reading would need the current time and would make
  two builds of one input disagree. On by default, in both modes, and drawn by
  nixdiag rather than d2, so `nixdiag check` compares it. Marker colour:
  `chartMark`.
- **Flake input graph.** A new `inputs.d2` diagram and `wiki/src/inputs.md`
  page, read straight from `flake.lock`. No eval and no realisation, so it
  costs nothing and behaves identically in both modes. The page lists every
  input with its source, revision and lock date, and separates two duplicate
  signals that need different reactions: a **diamond** (one repo locked at
  several revisions — a correctness risk, reported with the `follows` line
  that would fix it) and **redundancy** (one revision reached under several
  node names — only a wasted fetch). Forge `owner`/`repo` are compared
  case-insensitively; without that, `nixos/nixpkgs` sitting beside
  `NixOS/nixpkgs` — a real and easily hit diamond — goes unreported. A flake
  with no lock simply gets no input page.
- **Annotation engine.** Topology semantics now come from `#:` comment lines in
  your own module files rather than from built-in knowledge of any particular
  stack. Statements: role, `expose`, `->` / `<-`, `name`, `scope`, `unit`.
  Parsed render-side with `rnix`, resolved against the evaluated facts, so edge
  targets reference real state instead of strings. Malformed lines and dangling
  targets are hard errors.
- **Domain map.** `<sub>@<key>` in any fqdn position takes its suffix from
  `--domain KEY=DOMAIN`, flake `nixdiag.domains` or `mkDocs.domains`, so private
  domains stay out of public repo source.
- **Grammar editions.** `nixdiag --version` now reports the annotation grammar
  edition the binary implements. Declare the edition your files are written
  against with `--grammar N`, flake `nixdiag.grammar` or `mkDocs.grammar`;
  unset keeps zero-config zero-config. Declaring a *newer* edition than the
  binary implements is fatal and names both numbers; declaring an *older* one
  enters compatibility mode.
- **Deprecation channel.** Retired spellings are rewritten to their replacement
  and reported as `file:line: #: <old> deprecated since X.Y, use #: <new>`,
  and removed only at an edition bump. `--deny deprecated` (flake
  `nixdiag.deny`, `mkDocs.deny`) promotes those warnings to errors for
  consumers who want CI red immediately. Nothing is deprecated in grammar 1.
- **Diagram styling.** `--theme dark|light`, `--background`, repeatable
  `--color NAME=#HEX`, mirrored as `mkDocs` `theme` / `background` / `colors`.
- `mkDocs` gained `indexPage`, `bookToml` and `extraAssets`.

### Changed

- Default output gained `inputs.d2`, `inputs.svg`, `wiki/src/inputs.md` and
  `wiki/src/inputs-timeline.svg` plus a SUMMARY entry. Mode A consumers who
  commit `docs/` should run `nixdiag gen` once after upgrading; mode B
  consumers need do nothing.
- **Facts schema 1 → 2.** The projection now reads only quasi-frozen,
  stack-agnostic surfaces: enabled services and programs with their defining
  files, firewall ports, users, platform, `stateVersion`, package count. The
  two per-kind projections collapsed into a single generic
  `nix/projections/core.nix`.
- A schema mismatch now names both nixdiag versions, not just the two numbers —
  the actionable fact is a `lib` and a binary from different revisions.
- `nixdiag check` hints that a cosmetic output change is expected after an
  upgrade, and points at this file.
- A d2 compile failure is a hard render error (was a silent message on stderr).
- **The packaged binary wraps a `d2` built without image support**, cutting its
  runtime closure from 2308 MiB / 363 paths to 228 MiB / 69. d2's image support
  exists only for PNG export and pulls `playwright-driver.browsers` — Chromium,
  Firefox and WebKit — which nixdiag never launches: it only ever runs
  `d2 --layout elk in.d2 out.svg`. Rendered SVGs are byte-identical either way.
  Nothing changes for anyone who already has d2 on PATH, since the wrapper uses
  `--suffix`.

### Removed

- All service-specific extraction and the Rust topology heuristics: nginx
  upstream resolution, the headscale address book, and the
  tailscale/prometheus/beszel fields of schema 1. Each was a permanent promise
  to track nixpkgs option shapes; annotations record intent instead, which
  survives upstream refactors.

[kac]: https://keepachangelog.com/en/1.1.0/
[semver]: https://semver.org/spec/v2.0.0.html
