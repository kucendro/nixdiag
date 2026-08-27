//! What a set of annotations adds up to: nodes, their payloads, the edges
//! between them, and the named endpoints they front. Purely descriptive —
//! parsing lives in `stmt`, resolution against the facts in `resolve`.

use indexmap::IndexMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Public,
    Mesh,
    Lan,
}

impl Scope {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "public" => Some(Scope::Public),
            "mesh" => Some(Scope::Mesh),
            "lan" => Some(Scope::Lan),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Scope::Public => "public",
            Scope::Mesh => "mesh",
            Scope::Lan => "lan",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expose {
    pub port: u32,
    pub udp: bool,
    pub scope: Option<Scope>,
    pub name: Option<String>,
}

/// One node's annotation payload (a host box or a service inside it).
#[derive(Debug, Default)]
pub struct NodeInfo {
    pub role: Option<String>,
    pub scope: Option<Scope>,
    pub exposes: Vec<Expose>,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Endpoint {
    Host(String),
    Unit(String, String),
    Internet,
    Lan,
}

#[derive(Debug)]
pub struct Edge {
    pub from: Endpoint,
    pub to: Endpoint,
    pub label: String,
}

/// A named endpoint from `name=` on an edge: the annotated node fronts
/// `name` for `target` (an Endpoints page row, not a diagram element).
#[derive(Debug)]
pub struct NamedEndpoint {
    pub name: String,
    pub port: Option<u32>,
    pub node: Endpoint,
    pub target: Endpoint,
}

#[derive(Debug, Default)]
pub struct Model {
    pub hosts: IndexMap<String, NodeInfo>,
    pub units: IndexMap<(String, String), NodeInfo>,
    pub edges: Vec<Edge>,
    pub named: Vec<NamedEndpoint>,
    /// Total parsed statements — zero triggers the getting-started hint.
    pub total: usize,
}

impl Model {
    /// A node's declared scope: its own, else its host's.
    pub fn node_scope(&self, host: &str, unit: Option<&str>) -> Option<Scope> {
        unit.and_then(|u| {
            self.units
                .get(&(host.to_string(), u.to_string()))
                .and_then(|i| i.scope)
        })
        .or_else(|| self.hosts.get(host).and_then(|i| i.scope))
    }

    /// Effective scope of an expose: its own, else the node's, else the host's.
    pub fn effective_scope(&self, host: &str, unit: Option<&str>, e: &Expose) -> Option<Scope> {
        e.scope.or_else(|| self.node_scope(host, unit))
    }
}
