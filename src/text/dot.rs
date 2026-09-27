pub mod inputs {
    pub const ROOT: &str = "this flake";

    pub fn flagged(name: &str, rev: &str) -> String {
        format!("{name} {rev}")
    }
}
