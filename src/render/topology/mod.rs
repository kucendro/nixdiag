mod cloud;
mod host;
mod networks;
mod overview;

pub use host::HostBoard;
pub use networks::Networks;
pub use overview::Overview;

pub use cloud::{Cloud, Net};

use super::d2::Class;
use crate::facts::{Facts, Host, Kind};
use crate::text::d2::topology as t;
use crate::topology::{Connection, Endpoint, Model, Network};
use indexmap::IndexMap;
use itertools::Itertools;
use std::collections::BTreeSet;

pub enum Target<'a> {
    Node(&'a str, Option<&'a str>),
    Net(Cloud<'a>),
}

impl<'a> Target<'a> {
    fn of(e: &'a Endpoint, nets: &'a [Network]) -> Self {
        match e {
            Endpoint::Unit(h, u) => Target::Node(h, Some(u)),
            Endpoint::Host(h) => Target::Node(h, None),
            Endpoint::Internet => Target::Net(Cloud::Net(Net::Internet)),
            Endpoint::Network(id) => Target::Net(
                nets.iter()
                    .find(|n| n.id() == *id)
                    .map_or(Cloud::Net(Net::Lan), Cloud::Network),
            ),
        }
    }

    fn host(&self) -> Option<&'a str> {
        match self {
            Target::Node(h, _) => Some(h),
            Target::Net(_) => None,
        }
    }

    fn path(&self) -> Vec<String> {
        match self {
            Target::Node(h, u) => [Some(*h), *u]
                .into_iter()
                .flatten()
                .map(String::from)
                .collect(),
            Target::Net(c) => vec![c.key()],
        }
    }
}

pub struct Unit<'a> {
    pub name: &'a str,
    pub role: Option<String>,
    pub infra: bool,
    pub ports: BTreeSet<(u32, bool)>,
    pub description: Option<&'a str>,
}

impl Unit<'_> {
    fn ports(&self) -> Option<String> {
        let joined = self
            .ports
            .iter()
            .map(|(p, udp)| t::port(*p, *udp))
            .join(t::PORTS_SEP);
        (!joined.is_empty()).then_some(joined)
    }

    fn class(&self) -> Class {
        if self.infra {
            Class::Infra
        } else {
            Class::App
        }
    }
}

fn label(s: &str) -> Option<&str> {
    (!s.is_empty()).then_some(s)
}

pub struct Flow<'a> {
    pub from: Target<'a>,
    pub to: Target<'a>,
    pub label: &'a str,
}

pub struct Ingress<'a> {
    pub net: Cloud<'a>,
    pub node: Target<'a>,
    pub label: String,
}

pub struct View<'a> {
    pub facts: &'a Facts,
    pub hosts: IndexMap<&'a str, IndexMap<&'a str, Unit<'a>>>,
    pub ingress: Vec<Ingress<'a>>,
    pub networks: &'a [Network],
    connections: &'a [Connection],
}

fn icon(h: &Host) -> &'static str {
    match h {
        Host::Nixos(_) => t::NIXOS_ICON,
        Host::Darwin(_) => t::DARWIN_ICON,
    }
}

impl<'a> View<'a> {
    pub fn new(facts: &'a Facts, model: &'a Model) -> Self {
        let mut hosts: IndexMap<&str, IndexMap<&str, Unit>> = IndexMap::new();
        for (host, f) in &facts.hosts {
            let units = f.topology().units.iter().map(|(name, u)| {
                let unit = Unit {
                    name,
                    role: u.role.as_deref().map(t::role),
                    infra: matches!(u.kind, Some(Kind::Infra)),
                    ports: u.ports.iter().map(|p| (*p, false)).collect(),
                    description: u.description.as_deref(),
                };
                (name.as_str(), unit)
            });
            hosts.insert(host, units.collect());
        }
        for e in model.connections.iter().flat_map(|c| [&c.from, &c.to]) {
            if let Endpoint::Unit(h, u) = e {
                if let Some(units) = hosts.get_mut(h.as_str()) {
                    units.entry(u).or_insert_with(|| Unit {
                        name: u,
                        role: None,
                        infra: false,
                        ports: BTreeSet::new(),
                        description: None,
                    });
                }
            }
        }
        for x in &model.exposed {
            let unit = x.unit.as_deref().and_then(|u| {
                let units = hosts.get_mut(x.host.as_str())?;
                units.get_mut(u)
            });
            if let Some(unit) = unit {
                unit.ports.insert((x.port, x.udp));
            }
        }
        let nets = &model.networks;
        let exposed = model.exposed.iter().flat_map(|x| {
            Cloud::around(nets, &x.host, x.scope)
                .into_iter()
                .map(|net| Ingress {
                    net,
                    node: Target::Node(&x.host, x.unit.as_deref()),
                    label: t::expose(x.name.as_deref(), Some(x.port), x.udp),
                })
        });
        let named = model.named.iter().flat_map(|ne| {
            let host = ne.node.host().unwrap_or_default();
            Cloud::around(nets, host, ne.scope)
                .into_iter()
                .map(|net| Ingress {
                    net,
                    node: Target::of(&ne.node, nets),
                    label: t::expose(Some(&ne.name), ne.port, false),
                })
        });
        View {
            facts,
            hosts,
            ingress: exposed.chain(named).collect(),
            networks: &model.networks,
            connections: &model.connections,
        }
    }

    pub fn flows(&self) -> impl Iterator<Item = Flow<'a>> {
        self.connections.iter().map(|c| Flow {
            from: Target::of(&c.from, self.networks),
            to: Target::of(&c.to, self.networks),
            label: &c.label,
        })
    }

    fn label(&self, host: &str) -> String {
        let icon = self.facts.hosts.get(host).map_or(t::NIXOS_ICON, icon);
        t::host(icon, host)
    }

    pub fn boards(&self) -> impl Iterator<Item = HostBoard<'_, 'a>> {
        self.hosts
            .iter()
            .filter(|(_, units)| !units.is_empty())
            .map(|(host, _)| HostBoard { view: self, host })
    }
}
