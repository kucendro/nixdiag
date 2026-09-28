use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Facts {
    pub schema: u32,
    pub hosts: IndexMap<String, Host>,
}

impl Facts {
    pub fn bare(&self) -> bool {
        self.hosts.values().all(|h| h.topology().is_empty())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Host {
    Nixos(Box<NixosHost>),
    Darwin(Box<DarwinHost>),
}

impl Host {
    pub fn as_nixos(&self) -> Option<&NixosHost> {
        match self {
            Host::Nixos(h) => Some(h),
            Host::Darwin(_) => None,
        }
    }

    pub fn base(&self) -> &HostBase {
        match self {
            Host::Nixos(h) => &h.base,
            Host::Darwin(h) => &h.base,
        }
    }

    pub fn units(&self) -> impl Iterator<Item = &EnabledUnit> {
        self.base().services.iter().chain(&self.base().programs)
    }

    pub fn topology(&self) -> &Topology {
        &self.base().topology
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HostBase {
    pub services: Vec<EnabledUnit>,
    pub programs: Vec<EnabledUnit>,
    pub description: Option<String>,
    pub topology: Topology,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NixosHost {
    pub platform: String,
    pub state_version: String,
    pub tcp: Vec<u32>,
    pub udp: Vec<u32>,
    pub users: Vec<String>,
    pub pkg_count: u64,
    #[serde(flatten)]
    pub base: HostBase,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DarwinHost {
    pub casks: Vec<String>,
    pub daemons: Vec<String>,
    pub user_agents: Vec<String>,
    #[serde(flatten)]
    pub base: HostBase,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EnabledUnit {
    pub name: String,
    pub files: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Topology {
    pub role: Option<String>,
    pub scope: Option<Scope>,
    pub names: Vec<String>,
    pub expose: Vec<Expose>,
    pub units: IndexMap<String, Unit>,
}

impl Topology {
    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
            && self.expose.is_empty()
            && self.names.is_empty()
            && self.role.is_none()
    }

    pub fn scope_of(&self, unit: Option<&str>) -> Option<Scope> {
        unit.and_then(|u| self.units.get(u).and_then(|i| i.scope))
            .or(self.scope)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Unit {
    pub role: Option<String>,
    pub kind: Option<Kind>,
    pub scope: Option<Scope>,
    pub description: Option<String>,
    pub names: Vec<String>,
    pub ports: Vec<u32>,
    pub expose: Vec<Expose>,
    pub connections: Vec<Connection>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Expose {
    pub port: u32,
    pub udp: bool,
    pub scope: Option<Scope>,
    pub name: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Connection {
    pub to: String,
    pub label: String,
    pub name: Option<String>,
    pub port: Option<u32>,
    pub scope: Option<Scope>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Public,
    Mesh,
    Lan,
}

impl Scope {
    pub fn label(self) -> &'static str {
        match self {
            Scope::Public => "public",
            Scope::Mesh => "mesh",
            Scope::Lan => "lan",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Infra,
    App,
}
