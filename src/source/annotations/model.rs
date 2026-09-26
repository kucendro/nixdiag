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
    pub total: usize,
}

impl Model {
    pub fn node_scope(&self, host: &str, unit: Option<&str>) -> Option<Scope> {
        unit.and_then(|u| {
            self.units
                .get(&(host.to_string(), u.to_string()))
                .and_then(|i| i.scope)
        })
        .or_else(|| self.hosts.get(host).and_then(|i| i.scope))
    }

    pub fn effective_scope(&self, host: &str, unit: Option<&str>, e: &Expose) -> Option<Scope> {
        e.scope.or_else(|| self.node_scope(host, unit))
    }
}
