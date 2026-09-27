pub const ABOUT: &str = "Static infrastructure docs | Nixdiag";
pub const FACTS: &str = "facts.json path, or - for stdin";
pub const REPO: &str = "Repo source";
pub const CLOSURES: &str =
    "closures.json from `mkDocs { closures = true; }`, adding the Closures page";
pub const OUT: &str = "Output directory";
pub const TITLE: &str = "Wiki title";
pub const DEFAULT_TITLE: &str = "Nixdiag wiki";
pub const EXTRA_PAGE: &str = "Extra hand-written wiki page as TITLE=FILE;";
pub const EXTRA_LINK: &str =
    "SUMMARY entry as TITLE=NAME.md for a page written into wiki/src by another tool; repeatable";
pub const NO_SVG: &str = "Skip SVG rendering (d2)";
pub const THEME: &str = "Color theme";
pub const BACKGROUND: &str = "Diagram canvas fill";
pub const COLOR: &str = "Palette override as NAME=#HEX (names: the vars block in the d2 output, plus chartShared/chartPartial/chartUnique/chartInk/chartMuted/chartTrack for the SVG charts); repeatable";
