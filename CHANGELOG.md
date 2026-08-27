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

### Removed

- All service-specific extraction and the Rust topology heuristics: nginx
  upstream resolution, the headscale address book, and the
  tailscale/prometheus/beszel fields of schema 1. Each was a permanent promise
  to track nixpkgs option shapes; annotations record intent instead, which
  survives upstream refactors.

[kac]: https://keepachangelog.com/en/1.1.0/
[semver]: https://semver.org/spec/v2.0.0.html
