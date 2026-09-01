set shell := ["bash", "-euo", "pipefail", "-c"]

# List these recipes.
default:
    @just --list --unsorted

# Serve the hand-written docs site (site/) with live reload.
site: _site-assets
    mdbook serve site --open

# Render the example fleet with the working-tree binary and serve its wiki.
wiki: build
    #!/usr/bin/env bash
    set -euo pipefail
    facts="$(nix build .#fixture-facts --no-link --print-out-paths)"
    rm -rf .dev/docs
    ./target/debug/nixdiag render \
      --facts "$facts" --repo tests/fixture \
      --closures tests/fixture/closures.json \
      --domain ts=ts.example --title 'Example fleet' \
      --out .dev/docs
    mdbook serve .dev/docs/wiki --open

_site-assets:
    cp -f assets/topology.svg assets/modules.svg tests/reference/closures.svg site/src/

# Build the binary into target/debug.
build:
    cargo build

# What CI runs, ordered so the cheapest thing fails first.
check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test
    nix flake check

# Refresh tests/reference/ from the fixture, then show what moved.
snapshots:
    #!/usr/bin/env bash
    set -euo pipefail
    docs="$(nix build .#fixture-docs --no-link --print-out-paths)"
    closures="$(nix build .#fixture-docs-closures --no-link --print-out-paths)"
    cp --no-preserve=mode "$docs"/{topology,modules,inputs}.d2 tests/reference/
    cp --no-preserve=mode "$docs"/wiki/src/{hosts,endpoints,inputs}.md tests/reference/
    cp --no-preserve=mode "$closures"/wiki/src/closures.md tests/reference/
    cp --no-preserve=mode "$closures"/wiki/src/closures*.svg tests/reference/
    git diff --stat -- tests/reference/

# Re-render the README and site diagrams from tests/reference/*.d2.
assets:
    #!/usr/bin/env bash
    set -euo pipefail
    for d in topology modules; do
      d2 --layout elk --theme 200 "tests/reference/$d.d2" "assets/$d.svg"
    done
    git diff --stat -- assets/
