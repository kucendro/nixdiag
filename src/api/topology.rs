use super::Meta;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
pub struct Topology {
    pub meta: Meta,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub endpoints: Vec<EndpointRow>,
}

/// `id` is spelled the way the user writes it in `#: -> host/service`, not
/// the way d2 needs it — `util::sanitize` is lossy and exists for d2 alone.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Node {
    pub id: String,
    pub host: String,
    pub unit: Option<String>,
    pub role: Option<String>,
    pub scope: Option<String>,
    pub exposes: Vec<Expose>,
    pub names: Vec<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Expose {
    pub port: u32,
    pub protocol: &'static str,
    pub scope: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct EndpointRow {
    pub name: String,
    pub port: Option<u32>,
    pub scope: Option<String>,
    /// The node serving this endpoint.
    pub node: String,
    /// Present when the row came from `name=` on an edge.
    pub target: Option<String>,
}
