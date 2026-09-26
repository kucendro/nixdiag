use super::Meta;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
pub struct Inputs {
    pub meta: Meta,
    pub root: String,
    pub nodes: Vec<InputNode>,
    pub edges: Vec<InputEdge>,
    pub duplicates: Vec<Duplicate>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InputNode {
    pub name: String,
    pub source: String,
    pub rev: Option<String>,
    /// A fixed integer in `flake.lock`, never a clock read. Absent for a
    /// `path:` input, which has no date to place on a scale.
    pub last_modified: Option<i64>,
    /// Declared by the root flake, so `nix flake update` moves it. Everything
    /// else moves only when its parent does.
    pub direct: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct InputEdge {
    pub from: String,
    pub to: String,
    pub input: String,
    /// A `follows` *removes* a duplicate rather than adding an input.
    pub follows: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Duplicate {
    pub source: String,
    pub identity: String,
    /// One repo at several revisions — a correctness risk, unlike plain
    /// redundancy, which is one revision under several node names.
    pub diamond: bool,
    pub revisions: Vec<RevGroup>,
    /// Only suggested when the root actually has an input to point at.
    pub follows_target: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RevGroup {
    pub rev: String,
    pub nodes: Vec<String>,
}
