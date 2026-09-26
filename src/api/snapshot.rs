use super::Meta;
use schemars::JsonSchema;
use serde::Serialize;

/// The small document history accumulates. A few hundred bytes plus one
/// number per host: a trend means fetching many of these, so it is
/// deliberately not a copy of everything above.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Snapshot {
    pub meta: Meta,
    pub revision: Option<Revision>,
    pub totals: Totals,
}

/// Supplied by the caller, never discovered — `render` shells out to no git
/// and reads no clock.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Revision {
    pub id: String,
    pub dirty: bool,
    pub time: Option<i64>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Totals {
    pub hosts: usize,
    pub nixos_hosts: usize,
    pub darwin_hosts: usize,
    pub services: usize,
    pub programs: usize,
    pub ports: PortTotals,
    pub packages: u64,
    pub annotations: AnnotationTotals,
    pub inputs: Option<InputTotals>,
    pub closures: Option<ClosureTotals>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PortTotals {
    pub tcp: usize,
    pub udp: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AnnotationTotals {
    pub statements: usize,
    pub nodes: usize,
    pub edges: usize,
    pub endpoints: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InputTotals {
    pub total: usize,
    pub direct: usize,
    pub diamonds: usize,
    pub redundant: usize,
    pub oldest: Option<i64>,
    pub newest: Option<i64>,
    /// Oldest to newest, in days. Lock arithmetic, not a clock read.
    pub span_days: Option<i64>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClosureTotals {
    pub measured: usize,
    pub deduplicated_bytes: u64,
    pub naive_sum_bytes: u64,
    /// host -> total bytes, so a trend line needs only this one file.
    pub hosts: std::collections::BTreeMap<String, u64>,
}
