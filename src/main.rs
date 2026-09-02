//! nixdiag
//!
//!   `eval`   — mode A: run the projection over a flake, producing `facts`
//!   `facts`  — the schema-versioned contract between extraction and rendering
//!   `api`    — the published wire format other people's code reads
//!   `source` — static analysis of the documented repo's own .nix files:
//!              repo-relative paths, doc comments, the import graph, and the
//!              `#:` annotations that carry all topology intent
//!   `render` — facts + annotations -> d2 diagrams, the mdBook wiki, the API
//!   `cli`    — argument parsing, option resolution, command bodies

mod api;
mod cli;
mod closures;
mod eval;
mod facts;
mod render;
mod source;
mod util;

fn main() -> anyhow::Result<()> {
    cli::run()
}
