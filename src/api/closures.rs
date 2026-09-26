use super::Meta;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
pub struct Closures {
    pub meta: Meta,
    pub fleet: Fleet,
    pub hosts: Vec<HostClosure>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Fleet {
    pub measured_hosts: usize,
    pub shared_bytes: u64,
    pub shared_paths: usize,
    pub deduplicated_bytes: u64,
    pub deduplicated_paths: usize,
    /// What the hosts would cost if nothing were shared.
    pub naive_sum_bytes: u64,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HostClosure {
    pub name: String,
    /// A host can be unmeasured because it is darwin, or because it serves
    /// these docs and measuring it would be self-referential. Kept in the
    /// list either way: an omission would read as "not part of the fleet".
    pub measured: bool,
    pub total_bytes: Option<u64>,
    pub paths: Option<usize>,
    pub split: Option<Split>,
    /// Per package, not per store path — the only per-path identity is the
    /// path itself, which must never appear here. Untruncated, unlike the
    /// treemap, whose 24-tile cap is a drawing limit.
    pub packages: Vec<Package>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Split {
    pub shared_bytes: u64,
    pub partial_bytes: u64,
    pub unique_bytes: u64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Package {
    pub name: String,
    pub bytes: u64,
    /// How many measured hosts carry it.
    pub holders: usize,
}
