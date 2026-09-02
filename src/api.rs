//! The published API contract: every type here is a wire format someone
//! else's code reads.
//!
//! This is the fourth versioned surface (after the facts schema, the
//! annotation grammar and the package API) and it fails unlike any of them.
//! The facts schema is *fatal* on skew because both halves ship from one
//! flake, but a reader of this API is a third party that can only take what
//! it is handed — so nixdiag never validates it, and the documented contract
//! for readers is: tolerate unknown keys, and treat an unknown `schema` as
//! newer than you understand.
//!
//! Adding a key or an optional field does not bump `API_SCHEMA`. Removing or
//! renaming one does, with a CHANGELOG entry — a reworded table heading
//! breaks nobody's parser, a renamed key breaks every dashboard.

use crate::render::out::JSON_MARKER;
use schemars::JsonSchema;
use serde::Serialize;

/// URL prefix. A v2 lives beside v1 rather than replacing it, which is what
/// makes an incompatible change survivable for readers.
pub const API_VERSION: &str = "v1";

/// Bump only on a removal, rename, or change of meaning.
pub const API_SCHEMA: u32 = 1;

/// Carried by every document. `generator` holds the AUTO marker, which is
/// what lets the writer regenerate over its own output — JSON has no comment
/// to put it in.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub generator: &'static str,
    pub schema: u32,
    pub api: &'static str,
    pub nixdiag: &'static str,
    pub grammar: u32,
    pub facts_schema: u32,
}

impl Meta {
    pub fn new(grammar: u32) -> Self {
        Meta {
            generator: JSON_MARKER,
            schema: API_SCHEMA,
            api: API_VERSION,
            nixdiag: env!("CARGO_PKG_VERSION"),
            grammar,
            facts_schema: crate::facts::SCHEMA,
        }
    }
}

// ---------------------------------------------------------------- index

#[derive(Debug, Serialize, JsonSchema)]
pub struct Index {
    pub meta: Meta,
    pub endpoints: Vec<Link>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Link {
    pub path: String,
    pub description: &'static str,
}

// ---------------------------------------------------------------- hosts

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

// ------------------------------------------------------------- services

#[derive(Debug, Serialize, JsonSchema)]
pub struct Services {
    pub meta: Meta,
    pub services: Vec<ServiceEntry>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ServiceEntry {
    pub name: String,
    /// `service` or `program`.
    pub kind: &'static str,
    pub hosts: Vec<String>,
    /// Repo-relative, never store paths: Nix records a reference for every
    /// store path in a build output, so printing one would make the docs
    /// retain the closure it describes.
    pub files: Vec<String>,
}

// ------------------------------------------------------------- topology

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

// --------------------------------------------------------------- inputs

#[derive(Debug, Serialize, JsonSchema)]
pub struct Inputs {
    pub meta: Meta,
    pub root: String,
    pub nodes: Vec<InputNode>,
    pub edges: Vec<InputEdge>,
    pub duplicates: Vec<Duplicate>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InputNode {
    pub name: String,
    pub source: String,
    pub rev: Option<String>,
    /// A fixed integer in `flake.lock`, never a clock read. Absent for a
    /// `path:` input, which has no date to place on a scale.
    pub last_modified: Option<i64>,
    /// Declared by the root flake, so `nix flake update` moves it. Everything
    /// else moves only when its parent does.
    pub direct: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct InputEdge {
    pub from: String,
    pub to: String,
    pub input: String,
    /// A `follows` *removes* a duplicate rather than adding an input.
    pub follows: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Duplicate {
    pub source: String,
    pub identity: String,
    /// One repo at several revisions — a correctness risk, unlike plain
    /// redundancy, which is one revision under several node names.
    pub diamond: bool,
    pub revisions: Vec<RevGroup>,
    /// Only suggested when the root actually has an input to point at.
    pub follows_target: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RevGroup {
    pub rev: String,
    pub nodes: Vec<String>,
}

// ------------------------------------------------------------- closures

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

// ------------------------------------------------------------- snapshot

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
