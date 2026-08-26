# Nix visualization ideas

Distilled from the "Nix Visual Atlas · 21 plates" sketch (kept outside this
repo). A candidate second axis next to nixdiag: the atlas visualizes the **Nix
pipeline itself**, where nixdiag documents **infrastructure intent** across
hosts. Nothing here is committed work.

The discipline worth keeping from it: every plate is a question a person
actually asks, paired with the exact command that produces its data. No plate
is "a graph of the thing".

## The plates

| # | Plate | Question | Reads from |
|---|---|---|---|
| 1.1 | Eval flamegraph | where do those 40s of eval go? | `nix eval --eval-profiler=flamegraph` |
| 1.2 | Import graph, cost-weighted | which `.nix` files are expensive to pull in? | `NIX_SHOW_STATS=1`, import chain from `-vvv` |
| 1.3 | Laziness map | what got forced vs stayed a thunk (accidental strictness)? | `NIX_COUNT_CALLS=1`, forced set vs full attrset |
| 1.4 | Allocation over eval time | where does GC pressure spike in a slow eval? | `GC_PRINT_STATS=1`, RSS sampled over wall clock |
| 2.1 | Derivation DAG | what does this `.drv` actually depend on? | `nix-store -q --graph` |
| 2.2 | Rebuild blast radius | if this hash changes, what rebuilds? (tap a node) | `nix-store -q --referrers-closure` |
| 2.3 | Structural `.drv` diff | why did the output hash change between two builds? | `nix-diff` |
| 3.1 | Build Gantt, core lanes | where does the build serialize and leave cores idle? | `nix build --log-format internal-json` |
| 3.2 | Critical path | which single chain sets wall-clock time? | longest weighted path over `--json` build times |
| 3.3 | Cache hit/miss overlay | how much came from the cache vs built here? | `nix build --dry-run --json` |
| 4.1 | Closure size over time | is my system quietly getting heavier, and when did it jump? | `nix path-info -S` per system generation |
| 4.2 | Closure treemap, dominator-sized | what is 2.1 GB made of, and what would freeing it reclaim? | `nix-du` dominator sizes |
| 4.3 | Why-depends path | why is perl in my closure? | `nix why-depends` |
| 4.4 | Closure diff, what moved | this update grew the system 300 MB, where? | `nix store diff-closures` |
| 5.1 | Store growth timeline | how fast is `/nix/store` filling the disk between GCs? | `du -s` snapshots plus gc events |
| 5.2 | GC attribution, who pins what | which root holds 4 GB of dead-looking paths alive? | `nix-store --gc --print-dead`, grouped by nearest root |
| 5.3 | Generation timeline, git-log for your OS | what entered and left the system between boots? | `nix-env -p … --list-generations` |
| 6.1 | Flake input DAG with follows | what pulls what, and where do the follows dedup? | `nix flake metadata --json` |
| 6.2 | Input staleness | which locked inputs are overdue? | `flake.lock` lastModified vs now |
| 6.3 | Diamond detection | is nixpkgs pulled at two different revs at once? | group lock nodes by repo, flag >1 rev |
| 6.4 | Cross-commit tracking | how did this package's closure drift across nixpkgs? | `nix path-info -S` per rev |

Shared visual vocabulary across plates: built locally vs cached or substituted
vs dead or removed, plus unforced thunks and a highlighted path or primary
node.

## Triage

**About half are a good picture on an existing tool's output.** `nix-diff`
answers 2.3 in text, `nix-du` and `nix-tree` cover 4.2 and 4.3,
`nix store diff-closures` is 4.4, `nix-output-monitor` already taps the stream
3.1 needs, `nix-melt` reads the lock 6.1 wants. Not a criticism: the picture is
the missing part, and the data underneath is stable JSON. This is the tractable
half.

**The novel and hard half is Evaluation (1.1 to 1.4).** Nobody has a good
laziness map or an allocation-over-eval-time view, and that is where the pain
lives for large configs. The catch: the data sources are the least contractual
thing in Nix. `NIX_SHOW_STATS`, `NIX_COUNT_CALLS`, `-vvv` trace parsing and
`GC_PRINT_STATS` are debug output, not an API, and the eval profiler flag is
recent enough that the target Nix version matters. Same bar as the rest of this
repo: derivable from a stable generic surface, or not built.

**Best ratio of insight to effort:**

- 4.1 closure size over generations: trivial data, no existing chart, answers a
  question everyone has.
- 5.2 GC attribution: the "why is my disk full" question, and grouping dead
  paths by nearest root is real analysis rather than rendering.
- 6.3 diamond detection: pure `flake.lock` arithmetic, no nix invocation at
  all, catches a bug people hit constantly.

**The one with a moat:** 2.2 rebuild blast radius as an *interactive* plate.
Referrer closures are cheap to compute, and "tap a node, see what rebuilds" is
what no static graph delivers.
