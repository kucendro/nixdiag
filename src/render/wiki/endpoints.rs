//! The Endpoints page: what each node exposes and under which name, built
//! from `#: expose` and the opt-in `name=` on edges.

use super::super::out::{Out, MD_MARKER};
use crate::facts::Facts;
use crate::source::annotations::Model;
use anyhow::Result;
use std::path::Path;

pub(super) fn page_endpoints(
    out: &mut Out,
    src: &Path,
    facts: &Facts,
    model: &Model,
) -> Result<()> {
    let mut o: Vec<String> = vec![
        MD_MARKER.into(),
        "".into(),
        "# Endpoints".into(),
        "".into(),
        "Declared service endpoints across the fleet, from `#: expose` and \
         named `#: ->` annotations in the module files."
            .into(),
        "".into(),
        "| Endpoint | Port | Scope | Host | Service |".into(),
        "|---|---|---|---|---|".into(),
    ];
    // (endpoint, port, scope, host, service)
    let mut rows: Vec<(String, String, String, String, String)> = Vec::new();
    let mut push = |host: &str, unit: Option<&str>, info: &crate::source::annotations::NodeInfo| {
        for e in &info.exposes {
            let endpoint = e
                .name
                .clone()
                .unwrap_or_else(|| format!("{host}:{}", e.port));
            let port = format!("{}{}", e.port, if e.udp { "/udp" } else { "" });
            let scope = model
                .effective_scope(host, unit, e)
                .map(|s| s.label().to_string())
                .unwrap_or_else(|| "—".into());
            rows.push((
                endpoint,
                port,
                scope,
                host.to_string(),
                unit.unwrap_or("—").to_string(),
            ));
        }
    };
    for host in facts.hosts.keys() {
        if let Some(info) = model.hosts.get(host) {
            push(host, None, info);
        }
        for ((h, unit), info) in &model.units {
            if h == host {
                push(host, Some(unit), info);
            }
        }
    }
    // Named endpoints from `name=` on edges: the annotated node fronts the
    // fqdn, the edge target is the service behind it.
    use crate::source::annotations::Endpoint;
    for ne in &model.named {
        let (host, unit) = match &ne.node {
            Endpoint::Host(h) => (h.clone(), None),
            Endpoint::Unit(h, u) => (h.clone(), Some(u.clone())),
            _ => continue,
        };
        let scope = model
            .node_scope(&host, unit.as_deref())
            .map(|s| s.label().to_string())
            .unwrap_or_else(|| "—".into());
        let service = match &ne.target {
            Endpoint::Unit(_, u) => u.clone(),
            Endpoint::Host(h) => h.clone(),
            Endpoint::Internet => "internet".into(),
            Endpoint::Lan => "lan".into(),
        };
        rows.push((
            ne.name.clone(),
            ne.port.map(|p| p.to_string()).unwrap_or_else(|| "—".into()),
            scope,
            host,
            service,
        ));
    }
    rows.sort();
    for (endpoint, port, scope, host, service) in &rows {
        o.push(format!(
            "| `{endpoint}` | {port} | {scope} | {host} | {service} |"
        ));
    }
    if rows.is_empty() {
        o.push("| — | — | — | — | — |".into());
    }
    out.write_auto(&src.join("endpoints.md"), &o.join("\n"))
}
