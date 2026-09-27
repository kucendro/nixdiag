use crate::facts::Scope;

pub const INTERNET: &str = "internet";
pub const LAN: &str = "lan";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Endpoint {
    Host(String),
    Unit(String, String),
    Internet,
    Lan,
}

impl Endpoint {
    pub fn host(&self) -> Option<&str> {
        match self {
            Endpoint::Host(h) | Endpoint::Unit(h, _) => Some(h),
            Endpoint::Internet | Endpoint::Lan => None,
        }
    }
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

#[derive(Debug)]
pub struct Exposure {
    pub host: String,
    pub unit: Option<String>,
    pub name: Option<String>,
    pub port: u32,
    pub udp: bool,
    pub scope: Option<Scope>,
}

impl Exposure {
    pub fn node(&self) -> Endpoint {
        match &self.unit {
            Some(u) => Endpoint::Unit(self.host.clone(), u.clone()),
            None => Endpoint::Host(self.host.clone()),
        }
    }
}

#[derive(Debug, Default)]
pub struct Model {
    pub exposed: Vec<Exposure>,
    pub connections: Vec<Connection>,
    pub named: Vec<NamedEndpoint>,
}
