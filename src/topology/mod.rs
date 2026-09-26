mod model;
mod resolve;

pub use model::{Connection, Endpoint, Model, NamedEndpoint};

use crate::facts::{Facts, Scope};
use crate::text::{fill, messages as m};
use anyhow::{anyhow, Result};

pub fn build(facts: &Facts) -> Result<Model> {
    let book = resolve::Book::new(facts);
    let mut model = Model::default();
    for (host, h) in &facts.hosts {
        for (unit, u) in &h.topology().units {
            let node = Endpoint::Unit(host.clone(), unit.clone());
            for c in &u.connections {
                let to = resolve::target(facts, &book, host, &c.to).map_err(|reason| {
                    anyhow!(fill(
                        m::CONNECTION_ERROR,
                        &[
                            ("host", host),
                            ("unit", unit),
                            ("target", &c.to),
                            ("reason", &reason),
                        ],
                    ))
                })?;
                if let Some(name) = &c.name {
                    model.named.push(NamedEndpoint {
                        name: name.clone(),
                        port: c.port,
                        scope: c.scope,
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

pub fn scope_at(facts: &Facts, e: &Endpoint) -> Option<Scope> {
    let (host, unit) = match e {
        Endpoint::Host(h) => (h, None),
        Endpoint::Unit(h, u) => (h, Some(u.as_str())),
        _ => return None,
    };
    facts.hosts.get(host)?.topology().scope_of(unit)
}
