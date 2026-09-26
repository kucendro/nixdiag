use super::super::out::Out;
use super::page;
use crate::facts::{Expose, Facts};
use crate::text::fill;
use crate::text::wiki::{endpoints as t, NONE};
use crate::topology::{scope_at, Endpoint, Model};
use anyhow::Result;
use std::path::Path;

pub(super) fn page_endpoints(out: &Out, src: &Path, facts: &Facts, model: &Model) -> Result<()> {
    let mut rows: Vec<(String, String, String, String, String)> = Vec::new();
    for (host, h) in &facts.hosts {
        let topo = h.topology();
        let mut push = |unit: Option<&str>, e: &Expose| {
            let endpoint = e.name.clone().unwrap_or_else(|| {
                fill(t::UNNAMED, &[("host", host), ("port", &e.port.to_string())])
            });
            let port = format!("{}{}", e.port, if e.udp { t::UDP } else { "" });
            let scope = e
                .scope
                .or_else(|| topo.scope_of(unit))
                .map(|s| s.label().to_string())
                .unwrap_or_else(|| NONE.into());
            rows.push((
                endpoint,
                port,
                scope,
                host.clone(),
                unit.unwrap_or(NONE).to_string(),
            ));
        };
        for e in &topo.expose {
            push(None, e);
        }
        for (unit, u) in &topo.units {
            for e in &u.expose {
                push(Some(unit), e);
            }
        }
    }
    for ne in &model.named {
        let host = match &ne.node {
            Endpoint::Host(h) | Endpoint::Unit(h, _) => h.clone(),
            _ => continue,
        };
        let scope = ne
            .scope
            .or_else(|| scope_at(facts, &ne.node))
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
