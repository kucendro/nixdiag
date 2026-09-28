pub const BOOK: &str = "wiki/book.toml";
pub const SRC: &str = "wiki/src";
pub const CSS: &str = "d2.css";
pub const WIKI_CSS: &str = "wiki/d2.css";
pub const THEME: &str = "theme.d2";

pub mod diagram {
    pub const TOPOLOGY: &str = "topology";
    pub const INPUTS: &str = "inputs";

    pub fn modules(host: &str) -> String {
        format!("modules-{host}")
    }

    pub fn themed(stem: &str, theme: &str) -> String {
        format!("{stem}-{theme}.svg")
    }
}

pub mod page {
    pub const SUMMARY: &str = "SUMMARY.md";
    pub const INDEX: &str = "index.md";
    pub const ARCHITECTURE: &str = "architecture.md";
    pub const HOSTS: &str = "hosts.md";
    pub const SERVICES: &str = "services.md";
    pub const ENDPOINTS: &str = "endpoints.md";
    pub const INPUTS: &str = "inputs.md";
    pub const CLOSURES: &str = "closures.md";
}

pub mod chart {
    pub const TIMELINE: &str = "inputs-timeline.svg";
    pub const CLOSURES: &str = "closures.svg";

    pub fn host_closure(host: &str) -> String {
        format!("closures-{host}.svg")
    }
}
