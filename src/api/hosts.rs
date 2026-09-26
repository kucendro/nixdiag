use super::Meta;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
pub struct Hosts {
    pub meta: Meta,
    pub hosts: Vec<HostEntry>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HostEntry {
    pub name: String,
    pub kind: &'static str,
    pub platform: Option<String>,
    pub state_version: Option<String>,
    /// `environment.systemPackages` length; absent on darwin.
    pub packages: Option<u64>,
    pub users: Vec<String>,
    pub ports: Ports,
    /// From a host-level `#: <role>` / `#: scope`, if annotated.
    pub role: Option<String>,
    pub scope: Option<String>,
    /// Names only — `services.json` carries the defining files.
    pub services: Vec<String>,
    pub programs: Vec<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Ports {
    pub tcp: Vec<u32>,
    pub udp: Vec<u32>,
}
