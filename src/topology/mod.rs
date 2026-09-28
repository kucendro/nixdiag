mod firewall;
mod locations;
mod model;
mod networks;
mod resolve;
pub mod target;

pub use firewall::Finding;
pub use model::{Connection, Endpoint, Exposure, Model, NamedEndpoint, INTERNET, LAN};
pub use networks::{by_id, Network, Route};

use crate::facts::Facts;
use crate::text::messages::Fail;
use anyhow::Result;
use std::iter::once;

pub fn build(facts: &Facts) -> Result<Model> {
    let nets = networks::build(facts)?;
    let locations = locations::build(facts, &nets);
    let mut model = Model {
        exposed: exposed(facts),
        ..Model::default()
    };
    for x in model.exposed.iter_mut().filter(|x| x.scope.is_none()) {
        x.via = firewall::reach(facts, &nets, &x.host, x.port, x.udp);
        x.scope = networks::widest(&nets, &x.via);
    }
    let book = resolve::Book::new(facts, &model.exposed);
    for (host, h) in &facts.hosts {
        let topo = h.topology();
        for (unit, u) in &topo.units {
            let node = Endpoint::Unit(host.clone(), unit.clone());
            for c in &u.connections {
                let here = locations.get(host).map(String::as_str);
                let to =
                    resolve::target(facts, &nets, &book, host, here, &c.to).map_err(|reason| {
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
                let port = target::Target::parse(&c.to).and_then(|t| t.port);
                let port = port.or_else(|| sole_port(facts, &to));
                model.connections.push(Connection {
                    from: node.clone(),
                    to,
                    label: c.label.clone(),
                    plane: c.plane,
                    port,
                });
            }
        }
    }
    let audit = firewall::Audit {
        facts,
        nets: &nets,
        exposed: &model.exposed,
        connections: &model.connections,
    };
    let findings = facts
        .hosts
        .keys()
        .map(|h| (h.clone(), audit.of(h)))
        .collect();
    model.findings = findings;
    model.locations = locations;
    model.networks = nets;
    Ok(model)
}

fn sole_port(facts: &Facts, to: &Endpoint) -> Option<u32> {
    let Endpoint::Unit(h, u) = to else {
        return None;
    };
    match facts
        .hosts
        .get(h)?
        .topology()
        .units
        .get(u)?
        .ports
        .as_slice()
    {
        [p] => Some(*p),
        _ => None,
    }
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
                via: Vec::new(),
            }));
        }
    }
    out
}

#[cfg(test)]
mod tests;
