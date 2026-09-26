# CLI

```sh
nixdiag render --facts facts.json --repo . --out docs
```

Renders from a `facts.json` that `lib.mkFacts` produced and never calls
`nix`. This is what `mkDocs` runs in the sandbox; `nix run
github:kucendro/nixdiag` builds it for the flake in the current directory.

| Flag                                  | Effect                                                                                        |
| ------------------------------------- | --------------------------------------------------------------------------------------------- |
| `--facts FILE`                        | facts, `-` reads stdin                                                                        |
| `--repo DIR`                          | the documented repo, for the module tree and `flake.lock`                                     |
| `--out DIR`                           | output directory                                                                              |
| `--closures FILE`                     | the `closures.json` that [`mkDocs { closures = true; }`](./build.md#closure-metrics) produces |
| `--title "my wiki"`                   | book title, used when seeding `book.toml`                                                     |
| `--extra-page Runbooks=./runbooks.md` | copy a hand-written page in and link it, repeatable                                           |
| `--extra-link Termux=termux.md`       | SUMMARY entry for a page another tool writes, repeatable                                      |
| `--no-svg`                            | write `.d2` only, skip d2                                                                     |
| `--theme light`                       | `light` or `dark`, default `dark`                                                             |
| `--background "#ffffff"`              | diagram canvas fill, default transparent                                                      |
| `--color public=#ff5555`              | palette override, repeatable, see [styling](./build.md#diagram-styling)                       |

SVG rendering needs `d2` on `PATH`; the packaged binary wraps it in.

```sh
nix run github:kucendro/nixdiag#audit
```

Checks the adapters against nixpkgs, see [Topology](./topology.md#audit).
