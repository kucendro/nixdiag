//! nixdiag — static infrastructure docs from any Nix flake.
//!
//! The pipeline, and the module tree that mirrors it:
//!   `eval`   — mode A: run the projection over a flake, producing `facts`
//!   `facts`  — the schema-versioned contract between extraction and rendering
//!   `source` — static analysis of the documented repo's own .nix files:
//!              repo-relative paths, doc comments, the import graph, and the
//!              `#:` annotations that carry all topology intent
//!   `render` — facts + annotations -> d2 diagrams and the mdBook wiki
//!   `cli`    — argument parsing, option resolution, command bodies

mod cli;
mod eval;
mod facts;
mod render;
mod source;
mod util;

fn main() -> anyhow::Result<()> {
    cli::run()
}
