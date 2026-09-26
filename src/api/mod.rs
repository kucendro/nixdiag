mod closures;
mod hosts;
mod inputs;
mod services;
mod snapshot;
mod topology;

pub use closures::*;
pub use hosts::*;
pub use inputs::*;
pub use services::*;
pub use snapshot::*;
pub use topology::*;

use crate::render::out::JSON_MARKER;
use schemars::JsonSchema;
use serde::Serialize;

pub const API_VERSION: &str = "v1";

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
