use super::Meta;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
pub struct Services {
    pub meta: Meta,
    pub services: Vec<ServiceEntry>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ServiceEntry {
    pub name: String,
    /// `service` or `program`.
    pub kind: &'static str,
    pub hosts: Vec<String>,
    /// Repo-relative, never store paths: Nix records a reference for every
    /// store path in a build output, so printing one would make the docs
    /// retain the closure it describes.
    pub files: Vec<String>,
}
