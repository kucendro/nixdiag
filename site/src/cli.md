# CLI

```sh
nixdiag <command> [flags] [HOSTS…]
```

Positional host names restrict the run to those configurations. Default is
every host discovered in `nixosConfigurations` and `darwinConfigurations`.

## `gen`

Evaluate and render in one step. This is the everyday command.

```sh
nixdiag gen --flake . --out docs
nixdiag gen luna sol
```

`--out` beats the flake's `nixdiag.out`, which beats `<flake>/docs`.

## `check`

Re-render to a temp dir and diff against the committed output. Exits non-zero
listing the stale files, so CI catches docs that no longer match the config.
SVGs are skipped, d2 output varies with the d2 version, sources are compared
instead.

```sh
nixdiag check --flake .
```

## `facts`

Evaluate only, print `facts.json` on stdout. Useful to inspect what the
projection sees, or to render on a machine that has the repo but not the
evaluated closure.

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

Flags override the flake's `nixdiag` output for the same setting.

## Requirements

`gen`, `facts` and `check` shell out to `nix`. `render` does not. SVG
rendering needs `d2` on `PATH`, and the packaged binary wraps both in.
