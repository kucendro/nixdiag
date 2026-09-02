//! `api/index.html` — the reference page.
//!
//! A shim, not a dependency. Scalar is not packaged in nixpkgs, and both Rust
//! crates that wrap it (`utoipa-scalar`, `scalar-doc`) are ~11 KB of exactly
//! this markup plus a `<script>` pointing at jsDelivr — which would mean the
//! page needs internet *when viewed*, so a mesh-only wiki would render blank
//! and every viewer's browser would call out to a CDN. `mkDocs` copies a
//! pinned bundle in beside this file as `scalar.js` instead, so the reference
//! works offline and nothing leaves the network it is served on.

use crate::render::out::MD_MARKER;

pub(super) fn page() -> String {
    format!(
        r#"{MD_MARKER}
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>nixdiag API</title>
  </head>
  <body>
    <script id="api-reference" data-url="./v1/openapi.json"></script>
    <script src="./scalar.js"></script>
  </body>
</html>"#
    )
}
