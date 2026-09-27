use std::fmt::Display;
use std::path::PathBuf;
use thiserror::Error;

pub const FACTS: &str = "facts.json";
pub const CLOSURES: &str = "closures.json";
pub const FACTS_PRODUCER: &str = "module";
pub const CLOSURES_PRODUCER: &str = "derivation";
pub const NO_TOPOLOGY: &str = "note: no topology facts — the diagram shows hosts and firewall ports only. Set nixdiag.units.<unit> in a module to draw the data flow.";
pub const NO_D2: &str = "(d2 binary not on PATH -- skipped SVG render)";

pub fn wrote(path: impl Display) -> String {
    format!("wrote {path}")
}

pub fn lock_version(version: u32, expected: u32) -> String {
    format!("note: flake.lock is version {version}, expected {expected} — reading it anyway")
}

pub fn lock_unreadable(error: impl Display) -> String {
    format!("  ! flake.lock is not readable as a lock file, skipping: {error}")
}

#[derive(Debug, Error)]
pub enum Fail {
    #[error("reading {}", .0.display())]
    Reading(PathBuf),
    #[error("writing {}", .0.display())]
    Writing(PathBuf),
    #[error("parsing {0}")]
    Parsing(&'static str),
    #[error("{file} declares schema {found}, but nixdiag {} implements schema {expected} — the {producer} that produced it comes from a different nixdiag revision; pin `lib` and the binary to the same one", env!("CARGO_PKG_VERSION"))]
    Schema {
        file: &'static str,
        producer: &'static str,
        found: u32,
        expected: u32,
    },
    #[error("--extra-page {0}: source has no file name")]
    ExtraPageNoName(String),
    #[error("--extra-page {title}: {} not found", .path.display())]
    ExtraPageMissing { title: String, path: PathBuf },
    #[error("{host}/{unit} -> {target}: {reason}")]
    Connection {
        host: String,
        unit: String,
        target: String,
        reason: Unresolved,
    },
    #[error("d2 render of {0}.d2 failed: {1}")]
    D2(String, std::io::Error),
    #[error("d2 render of {0}.d2 failed:\n{1}")]
    D2Output(String, String),
}

#[derive(Debug, Error)]
pub enum Unresolved {
    #[error("unknown host `{0}`")]
    Host(String),
    #[error("`{unit}` is not enabled on `{host}`")]
    NotEnabled { unit: String, host: String },
    #[error("`{0}` runs on several hosts ({1}); use host/{0}")]
    AmbiguousUnit(String, String),
    #[error("`{0}` is served by several units; use host/unit")]
    AmbiguousName(String),
    #[error("nothing on `{0}` listens on port {1}")]
    NoPort(String, String),
    #[error("not a host, a unit, a declared name, a URL, `internet` or `lan`")]
    Unknown,
}

#[derive(Debug, Error)]
pub enum BadArg {
    #[error("needs text on both sides of =")]
    Pair,
    #[error("unknown color {0}; palette: {1}")]
    Color(String, String),
}
