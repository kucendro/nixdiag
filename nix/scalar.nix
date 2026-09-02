# The Scalar API reference bundle, vendored.
#
# Scalar is not packaged in nixpkgs, and the two Rust crates that wrap it
# (`utoipa-scalar`, `scalar-doc`) are ~11 KB of markup plus a `<script>`
# pointing at jsDelivr — so taking one would mean the reference page needs
# internet *when viewed*. A wiki served on a mesh with no route out would
# render blank, and every viewer's browser would call a CDN. Fetching it here
# instead is a fixed-output derivation: pure, hashed, offline at view time.
#
# The version is pinned deliberately. `@latest` would break the hash the
# moment upstream publishes, which is the failure a pin exists to prevent.
# Bumping it is a deliberate act with a CHANGELOG line, like any other input.
{ fetchurl }:
let
  version = "1.67.0";
in
fetchurl {
  name = "scalar-api-reference-${version}.js";
  url = "https://cdn.jsdelivr.net/npm/@scalar/api-reference@${version}/dist/browser/standalone.js";
  hash = "sha256-0VDm2ewzMGLLFYcHBLuetuxvqZzj/lsWSlO8BHDoOO4=";
}
