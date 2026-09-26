pub const WROTE: &str = "wrote {path}";
pub const READING: &str = "reading {path}";
pub const PARSING_FACTS: &str = "parsing facts.json";
pub const PARSING_CLOSURES: &str = "parsing closures.json";
pub const BAD_PAIR: &str = "{flag} expects {shape}, got: {value}";
pub const UNKNOWN_COLOR: &str = "unknown color {name}; palette: {palette}";
pub const EXTRA_PAGE_NO_NAME: &str = "--extra-page {title}: source has no file name";
pub const EXTRA_PAGE_MISSING: &str = "--extra-page {title}: {path} not found";
pub const HAND_WRITTEN: &str = "refusing to overwrite {path}: it exists but has no '{marker}' marker (looks hand-written); delete or move it first";
pub const SCHEMA_MISMATCH: &str = "facts.json declares schema {found}, but nixdiag {version} implements schema {expected} — the projection that produced these facts comes from a different nixdiag revision; pin `lib` and the binary to the same one";
pub const CLOSURES_SCHEMA_MISMATCH: &str = "closures.json declares schema {found}, but nixdiag {version} implements schema {expected} — the derivation that produced it comes from a different nixdiag revision; pin `lib` and the binary to the same one";
pub const ERROR: &str = "error: {message}";
pub const WARNING: &str = "warning: {message}";
pub const ANNOTATION_ERRORS: &str = "{count} annotation error(s)";
pub const NO_ANNOTATIONS: &str = "note: no `#:` annotations found — the topology shows hosts and firewall ports only. Annotate your modules to draw the data flow (see the Annotations section of the nixdiag README).";
pub const NO_D2: &str = "(d2 binary not on PATH -- skipped SVG render)";
pub const D2_FAILED: &str = "d2 render of {stem}.d2 failed: {error}";
pub const D2_FAILED_OUTPUT: &str = "d2 render of {stem}.d2 failed:\n{stderr}";
pub const LOCK_VERSION: &str =
    "note: flake.lock is version {version}, expected 7 — reading it anyway";
pub const LOCK_UNREADABLE: &str =
    "  ! flake.lock is not readable as a lock file, skipping: {error}";
