# CLI

```sh
nixdiag <command> [flags] [HOSTS…]
```

Positional host names restrict the run to those configurations. Default is
every host discovered in `nixosConfigurations` and `darwinConfigurations`.

## `gen`

Evaluate and render in one step.

```sh
nixdiag gen --flake . --out docs
nixdiag gen luna sol
```

`--out` beats the flake's `nixdiag.out`, which beats `<flake>/docs`.

## `check`

Re-render to a temp dir and diff against the committed output, exiting non-zero
with the stale files listed. d2-produced SVGs are skipped — their bytes move
with the d2 version — and the `.d2` sources compared instead.

```sh
nixdiag check --flake .
```

## `facts`

Evaluate only, print `facts.json` on stdout — what the projection sees.

```sh
nixdiag facts --flake . > facts.json
```

## `render`

Render from an existing `facts.json`. Needs the repo source for annotations
and doc comments, but never calls `nix`. This is what the `mkDocs` derivation
runs in the sandbox.

```sh
nixdiag render --facts facts.json --repo . --out docs
nixdiag facts | nixdiag render --facts - --repo .
```

## Flags

`--flake DIR` on `facts`, `gen`, `check`, default `.`.
`--repo DIR` and `--facts FILE` on `render`, `-` reads stdin.
`--closures FILE` on `render`, taking the `closures.json` that
[`mkDocs { closures = true; }`](./build.md#closure-metrics) produces. `gen` and
`check` accept the flag only to tell you it cannot work there: closure sizes
need every host's system built, which only a derivation can express purely.
`--out DIR` on `render`, `gen`, `check`.

Render flags, accepted by `render`, `gen` and `check`:

| Flag | Effect |
|---|---|
| `--title "my wiki"` | book title, used when seeding `book.toml` |
| `--extra-page Runbooks=./runbooks.md` | copy a hand-written page in and link it, repeatable |
| `--extra-link Termux=termux.md` | SUMMARY entry for a page another tool writes, repeatable |
| `--no-svg` | write `.d2` only, skip d2 |
| `--theme light` | `light` or `dark`, default `dark` |
| `--background "#ffffff"` | diagram canvas fill, default transparent |
| `--color public=#ff5555` | palette override, repeatable, see [styling](./build.md#diagram-styling) |
| `--domain home=home.example.com` | `@key` suffix for annotation fqdns, repeatable |
| `--grammar 1` | annotation grammar edition your modules are written against |
| `--deny deprecated` | promote deprecation warnings to errors, repeatable |
| `--no-api` | skip the published [`api/`](./api.md) tree |
| `--revision REV` | revision recorded in `api/v1/snapshot.json`; never discovered |
| `--revision-time UNIX` | unix time of that revision |

Flags override the flake's `nixdiag` output for the same setting.

## Requirements

`gen`, `facts` and `check` shell out to `nix`. `render` does not. SVG
rendering needs `d2` on `PATH`, and the packaged binary wraps both in.
