mod model;
mod resolve;
pub mod target;

pub use model::{Connection, Endpoint, Exposure, Model, NamedEndpoint, INTERNET, LAN};

use crate::facts::Facts;
use crate::text::messages::Fail;
use anyhow::Result;
use std::iter::once;

pub fn build(facts: &Facts) -> Result<Model> {
    let mut model = Model {
        exposed: exposed(facts),
        ..Model::default()
    };
    let book = resolve::Book::new(facts, &model.exposed);
    for (host, h) in &facts.hosts {
        let topo = h.topology();
        for (unit, u) in &topo.units {
            let node = Endpoint::Unit(host.clone(), unit.clone());
            for c in &u.connections {
                let to = resolve::target(facts, &book, host, &c.to).map_err(|reason| {
                    let (host, unit, target) = (host.clone(), unit.clone(), c.to.clone());
                    Fail::Connection {
                        host,
                        unit,
                        target,
                        reason,
                    }
                })?;
                if let Some(name) = &c.name {
                    model.named.push(NamedEndpoint {
                        name: name.clone(),
                        port: c.port,
                        scope: c.scope.or_else(|| topo.scope_of(Some(unit))),
                        node: node.clone(),
                        target: to.clone(),
                    });
                }
                model.connections.push(Connection {
                    from: node.clone(),
                    to,
                    label: c.label.clone(),
                });
            }
        }
    }
    Ok(model)
}

fn exposed(facts: &Facts) -> Vec<Exposure> {
    let mut out = Vec::new();
    for (host, h) in &facts.hosts {
        let topo = h.topology();
        let units = topo
            .units
            .iter()
            .map(|(u, i)| (Some(u.as_str()), &i.expose));
        for (unit, exposes) in once((None, &topo.expose)).chain(units) {
            out.extend(exposes.iter().map(|e| Exposure {
                host: host.clone(),
                unit: unit.map(Into::into),
                name: e.name.clone(),
                port: e.port,
                udp: e.udp,
                scope: e.scope.or_else(|| topo.scope_of(unit)),
            }));
        }
    }
    out
}
