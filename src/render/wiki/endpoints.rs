use super::{code, table, Wiki};
use crate::conf::files::page;
use crate::facts::Scope;
use crate::text::fill;
use crate::text::wiki::{endpoints as t, NONE};
use crate::topology::{Endpoint, INTERNET, LAN};
use anyhow::Result;

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
            return code(&self.endpoint);
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

    fn cells(self) -> [String; 5] {
        let port = self.port.map_or(NONE.into(), |p| p.to_string());
        let port = port + if self.udp { t::UDP } else { "" };
        [self.cell(), port, self.scope, self.host, self.service]
    }
}

fn scope(s: Option<Scope>) -> String {
    s.map_or(NONE, Scope::label).into()
}

pub(super) fn page_endpoints(w: &Wiki) -> Result<()> {
    let exposed = w.model.exposed.iter().map(|x| Row {
        endpoint: x.name.clone().unwrap_or_else(|| {
            fill(
                t::UNNAMED,
                &[("host", &x.host), ("port", &x.port.to_string())],
            )
        }),
        port: Some(x.port),
        udp: x.udp,
        scope: scope(x.scope),
        host: x.host.clone(),
        service: x.unit.as_deref().unwrap_or(NONE).into(),
        named: x.name.is_some(),
    });
    let named = w.model.named.iter().filter_map(|ne| {
        Some(Row {
            endpoint: ne.name.clone(),
            port: ne.port,
            udp: false,
            scope: scope(ne.scope),
            host: ne.node.host()?.into(),
            service: match &ne.target {
                Endpoint::Unit(_, s) | Endpoint::Host(s) => s.clone(),
                Endpoint::Internet => INTERNET.into(),
                Endpoint::Lan => LAN.into(),
            },
            named: true,
        })
    });
    let mut rows: Vec<Row> = exposed.chain(named).collect();
    rows.sort();
    let table = table(t::HEAD, rows.into_iter().map(Row::cells));
    w.page(page::ENDPOINTS, &[t::TITLE.to_string(), table])
}
