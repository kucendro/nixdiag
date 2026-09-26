use crate::facts::Scope;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    Host(String),
    Unit(String, String),
    Internet,
    Lan,
}

#[derive(Debug)]
pub struct Connection {
    pub from: Endpoint,
    pub to: Endpoint,
    pub label: String,
}

#[derive(Debug)]
pub struct NamedEndpoint {
    pub name: String,
    pub port: Option<u32>,
    pub scope: Option<Scope>,
    pub node: Endpoint,
    pub target: Endpoint,
}

#[derive(Debug, Default)]
pub struct Model {
    pub connections: Vec<Connection>,
    pub named: Vec<NamedEndpoint>,
}
