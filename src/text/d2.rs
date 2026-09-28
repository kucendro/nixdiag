pub const IMPORT: &str = "...@theme";

pub fn var(name: &str, value: &str) -> String {
    format!("  {name}: {value}")
}

pub fn vars(lines: &str) -> String {
    format!("vars: {{\n{lines}\n}}")
}

pub fn fill(color: &str) -> String {
    format!("style.fill: {color}")
}

pub mod modules {
    pub const SERVICE: &str = "service";
    pub const PROGRAM: &str = "program";
}

pub mod inputs {
    pub const ROOT: &str = "this flake";

    pub fn flagged(name: &str, rev: &str) -> String {
        format!("{name}\n{rev}")
    }
}
