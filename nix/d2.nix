# The d2 nixdiag actually needs: SVG only, no browser engines.
#
# `withImageSupport` exists solely for d2's PNG export, and it pulls
# `playwright-driver.browsers` — Chromium, Firefox and WebKit. nixdiag never
# asks for a raster; `render/d2.rs` only ever runs
# `d2 --layout elk in.d2 out.svg`. Leaving the option at its nixpkgs default
# cost 2.2 GiB: measured on x86_64-linux, the packaged binary's runtime closure
# was 2308 MiB across 363 paths, against 228 MiB across 69 without it. The
# same weight lands in every mode B docs build and in the devShell, which is
# why all three call sites go through here.
#
# Rendered SVGs are byte-identical either way (checked against all three
# reference diagrams and the committed assets/). `withImageSupport` is a named
# upstream argument, so if nixpkgs ever renames it `.override` throws at eval
# naming the argument — loud, not silent rot.
d2: d2.override { withImageSupport = false; }
