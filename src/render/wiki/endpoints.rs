use super::super::out::Out;
use super::page;
use crate::facts::{Expose, Facts};
use crate::text::fill;
use crate::text::wiki::{endpoints as t, NONE};
use crate::topology::{scope_at, Endpoint, Model};
use anyhow::Result;
use std::path::Path;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Row {
    endpoint: String,
    port: Option<u32>,
    udp: bool,
    scope: String,
    host: String,
    service: String,
    named: bool,
}

impl Row {
    fn cell(&self) -> String {
        if !self.named || self.udp {
            return fill(t::NAME, &[("name", &self.endpoint)]);
        }
        let (scheme, port) = match self.port {
            Some(80) => (t::HTTP, String::new()),
            Some(443) | None => (t::HTTPS, String::new()),
            Some(p) => (t::HTTP, format!(":{p}")),
        };
        fill(
            t::LINK,
            &[
                ("name", &self.endpoint),
                ("scheme", scheme),
                ("port", &port),
            ],
        )
    }

    fn line(&self) -> String {
        let port = format!(
            "{}{}",
            self.port
                .map(|p| p.to_string())
                .unwrap_or_else(|| NONE.into()),
            if self.udp { t::UDP } else { "" }
        );
        fill(
            t::ROW,
            &[
                ("endpoint", &self.cell()),
                ("port", &port),
                ("scope", &self.scope),
                ("host", &self.host),
                ("service", &self.service),
            ],
        )
    }
}

pub(super) fn page_endpoints(out: &Out, src: &Path, facts: &Facts, model: &Model) -> Result<()> {
    let mut rows = Vec::new();
    for (host, h) in &facts.hosts {
        let topo = h.topology();
        let mut push = |unit: Option<&str>, e: &Expose| {
            let endpoint = e.name.clone().unwrap_or_else(|| {
                fill(t::UNNAMED, &[("host", host), ("port", &e.port.to_string())])
            });
            let scope = e
                .scope
                .or_else(|| topo.scope_of(unit))
                .map(|s| s.label().to_string())
                .unwrap_or_else(|| NONE.into());
            rows.push(Row {
                endpoint,
                port: Some(e.port),
                udp: e.udp,
                scope,
                host: host.clone(),
                service: unit.unwrap_or(NONE).into(),
                named: e.name.is_some(),
            });
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
        rows.push(Row {
            endpoint: ne.name.clone(),
            port: ne.port,
            udp: false,
            scope,
            host,
            service,
            named: true,
        });
    }
    rows.sort();
    let mut lines: Vec<String> = rows.iter().map(Row::line).collect();
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
