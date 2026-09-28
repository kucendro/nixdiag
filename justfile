set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list --unsorted

site: _site-assets
    mdbook serve site --open

wiki: build
    #!/usr/bin/env bash
    set -euo pipefail
    facts="$(nix build .#fixture-facts --no-link --print-out-paths)"
    rm -rf .dev/docs
    ./target/debug/nixdiag \
      --facts "$facts" --repo tests/fixture \
      --closures tests/fixture/closures.json \
      --title 'Example fleet' --out .dev/docs
    mdbook serve .dev/docs/wiki --open

_site-assets:
    cp -f assets/topology-light.svg site/src/topology.svg
    cp -f assets/modules-jerry-light.svg site/src/modules.svg
    cp -f assets/closures-light.svg site/src/closures.svg
    cp -f "$(nix build .#adapters-page --no-link --print-out-paths)/adapters.md" site/src/adapters.md

build:
    cargo build

check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test
    nix flake check

snapshots:
    #!/usr/bin/env bash
    set -euo pipefail
    declare -A out
    out[docs]="$(nix build .#fixture-docs --no-link --print-out-paths)"
    out[closures]="$(nix build .#fixture-docs-closures --no-link --print-out-paths)"
    while read -r build path; do
      [ -n "$build" ] || continue
      install -Dm644 "${out[$build]}/$path" "tests/reference/$path"
    done < <(grep -v '^\s*#' tests/reference/MANIFEST)
    git diff --stat -- tests/reference/

assets:
    cp -f "$(nix build .#fixture-assets --no-link --print-out-paths)"/*.svg assets/
    chmod 644 assets/*.svg
    git diff --stat -- assets/
