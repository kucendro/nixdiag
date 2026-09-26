use super::super::out::Out;
use super::page;
use crate::facts::Facts;
use crate::source::annotations::{Endpoint, Model, NodeInfo};
use crate::text::fill;
use crate::text::wiki::{endpoints as t, NONE};
use anyhow::Result;
use std::path::Path;

pub(super) fn page_endpoints(
    out: &mut Out,
    src: &Path,
    facts: &Facts,
    model: &Model,
) -> Result<()> {
    let mut rows: Vec<(String, String, String, String, String)> = Vec::new();
    let mut push = |host: &str, unit: Option<&str>, info: &NodeInfo| {
        for e in &info.exposes {
            let endpoint = e.name.clone().unwrap_or_else(|| {
                fill(t::UNNAMED, &[("host", host), ("port", &e.port.to_string())])
            });
            let port = format!("{}{}", e.port, if e.udp { t::UDP } else { "" });
            let scope = model
                .effective_scope(host, unit, e)
                .map(|s| s.label().to_string())
                .unwrap_or_else(|| NONE.into());
            rows.push((
                endpoint,
                port,
                scope,
                host.to_string(),
                unit.unwrap_or(NONE).to_string(),
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
    for ne in &model.named {
        let (host, unit) = match &ne.node {
            Endpoint::Host(h) => (h.clone(), None),
            Endpoint::Unit(h, u) => (h.clone(), Some(u.clone())),
            _ => continue,
        };
        let scope = model
            .node_scope(&host, unit.as_deref())
            .map(|s| s.label().to_string())
            .unwrap_or_else(|| NONE.into());
        let service = match &ne.target {
            Endpoint::Unit(_, u) => u.clone(),
            Endpoint::Host(h) => h.clone(),
            Endpoint::Internet => t::INTERNET.into(),
            Endpoint::Lan => t::LAN.into(),
        };
        rows.push((
            ne.name.clone(),
            ne.port
                .map(|p| p.to_string())
                .unwrap_or_else(|| NONE.into()),
            scope,
            host,
            service,
        ));
    }
    rows.sort();
    let mut lines: Vec<String> = rows
        .iter()
        .map(|(endpoint, port, scope, host, service)| {
            fill(
                t::ROW,
                &[
                    ("endpoint", endpoint),
                    ("port", port),
                    ("scope", scope),
                    ("host", host),
                    ("service", service),
                ],
            )
        })
        .collect();
    if lines.is_empty() {
        lines.push(t::EMPTY.into());
    }
    page(
        out,
        &src.join("endpoints.md"),
        &[
            t::TITLE.to_string(),
            fill(t::TABLE, &[("rows", &lines.join("\n"))]),
        ],
    )
}
