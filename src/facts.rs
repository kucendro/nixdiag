//! The facts.json contract between extraction (Nix projections) and rendering.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Bump on any breaking change to this model or to nix/projections/.
pub const SCHEMA: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct Facts {
    pub schema: u32,
    pub hosts: IndexMap<String, Host>,
}

impl Facts {
    /// Canonical host order: nixos first, then darwin, each sorted by name.
    /// Mode A discovery and mode B's alphabetical toJSON both normalize to
    /// this, so the two modes render identical documents.
    pub fn normalize(&mut self) {
        self.hosts.sort_by(|k1, v1, k2, v2| {
            let rank = |h: &Host| match h {
                Host::Nixos(_) => 0,
                Host::Darwin(_) => 1,
            };
            rank(v1).cmp(&rank(v2)).then(k1.cmp(k2))
        });
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Host {
    Nixos(Box<NixosHost>),
    Darwin(DarwinHost),
}

impl Host {
    pub fn as_nixos(&self) -> Option<&NixosHost> {
        match self {
            Host::Nixos(h) => Some(h),
            Host::Darwin(_) => None,
        }
    }

    pub fn svc_count(&self) -> usize {
        match self {
            Host::Nixos(h) => h.services.len(),
            Host::Darwin(h) => h.daemons.len() + h.user_agents.len(),
        }
    }
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
    pub tailscale: bool,
    pub routes: Vec<String>,
    pub headscale: bool,
    pub headscale_port: u32,
    pub base_domain: String,
    pub policy_path: String,
    pub beszel_hub: bool,
    pub beszel_hub_port: u32,
    pub beszel_agent: bool,
    pub prometheus: bool,
    pub blackbox: bool,
    pub grafana: bool,
    pub prom_targets: Vec<String>,
    pub vhosts: Vec<Vhost>,
    pub services: Vec<EnabledUnit>,
    pub programs: Vec<EnabledUnit>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DarwinHost {
    pub casks: Vec<String>,
    pub daemons: Vec<String>,
    pub user_agents: Vec<String>,
    pub services: Vec<EnabledUnit>,
    pub programs: Vec<EnabledUnit>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Vhost {
    pub name: String,
    pub listen: Vec<String>,
    pub pass: Option<String>,
    pub extra: String,
}

/// A unit (service/program) that some module file enables.
/// `files` are raw store paths; repo-relative resolution happens at render time.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EnabledUnit {
    pub name: String,
    pub files: Vec<String>,
}
