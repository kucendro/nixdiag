//! The Data API page: how to read this wiki's contents as JSON.
//!
//! Written only when the `api/` tree was published, and it lists exactly the
//! documents that were written, so it can never advertise a missing endpoint.

use super::super::out::{Out, MD_MARKER};
use crate::api::API_VERSION;
use anyhow::Result;
use std::path::Path;

pub(super) fn page_api(
    out: &mut Out,
    src: &Path,
    has_inputs: bool,
    has_closures: bool,
) -> Result<()> {
    let v = API_VERSION;
    let mut o: Vec<String> = vec![
        MD_MARKER.into(),
        "".into(),
        "# Data API".into(),
        "".into(),
        "| Endpoint | Contents |".into(),
        "|---|---|".into(),
        format!("| `/api/{v}/index.json` | the endpoints this build published |"),
        format!("| `/api/{v}/hosts.json` | hosts, platform, open ports, users |"),
        format!("| `/api/{v}/services.json` | services this repo configures, and where |"),
        format!("| `/api/{v}/topology.json` | annotated nodes, edges and endpoints |"),
    ];
    if has_inputs {
        o.push(format!(
            "| `/api/{v}/inputs.json` | lock graph, dates and duplicates |"
        ));
    }
    if has_closures {
        o.push(format!(
            "| `/api/{v}/closures.json` | per-host sizes by package |"
        ));
    }
    o.push(format!(
        "| `/api/{v}/snapshot.json` | totals plus revision identity |"
    ));
    o.push(format!(
        "| `/api/{v}/openapi.json` | an OpenAPI 3.1 document describing all of the above |"
    ));
    o.push("".into());
    // Absolute, because that is how the vhost serves them: the book is the web
    // root and `/api/` a sibling location.
    o.push("Paths are relative to the site root.".into());
    out.write_auto(&src.join("api.md"), &o.join("\n"))
}
