use super::networks::{owner, Network};
use super::target::Target;
use super::{Endpoint, Exposure, INTERNET, LAN};
use crate::facts::Facts;
use crate::text::messages::Unresolved;
use ipnet::IpNet;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::net::IpAddr;
use url::Host;

struct Entry {
    node: Endpoint,
    port: Option<u32>,
}

pub struct Book(BTreeMap<String, Vec<Entry>>);

impl Book {
    pub fn new(facts: &Facts, exposed: &[Exposure]) -> Self {
        let mut book: BTreeMap<String, Vec<Entry>> = BTreeMap::new();
        let mut add = |name: &str, node: Endpoint, port: Option<u32>| {
            book.entry(name.to_string())
                .or_default()
                .push(Entry { node, port });
        };
        for x in exposed {
            if let Some(n) = &x.name {
                add(n, x.node(), Some(x.port));
            }
        }
        for (host, h) in &facts.hosts {
            let topo = h.topology();
            for n in &topo.names {
                add(n, Endpoint::Host(host.clone()), None);
            }
            for (unit, u) in &topo.units {
                let node = Endpoint::Unit(host.clone(), unit.clone());
                for n in &u.names {
                    add(n, node.clone(), None);
                }
                for c in &u.connections {
                    if let Some(n) = &c.name {
                        add(n, node.clone(), c.port);
                    }
                }
            }
        }
        Book(book)
    }

    fn lookup(&self, name: &str, port: Option<u32>) -> Result<Option<Endpoint>, Unresolved> {
        let Some(entries) = self.0.get(name) else {
            return Ok(None);
        };
        let on_port = entries.iter().filter(|e| port.is_some() && e.port == port);
        let mut nodes: Vec<&Endpoint> = on_port.map(|e| &e.node).unique().collect();
        if nodes.is_empty() {
            nodes = entries.iter().map(|e| &e.node).unique().collect();
        }
        match nodes.as_slice() {
            [n] => Ok(Some((*n).clone())),
            _ => Err(Unresolved::AmbiguousName(name.into())),
        }
    }
}

pub fn target(
    facts: &Facts,
    nets: &[Network],
    book: &Book,
    from: &str,
    target: &str,
) -> Result<Endpoint, Unresolved> {
    if target == INTERNET {
        return Ok(Endpoint::Internet);
    }
    if let Ok(cidr) = target.parse::<IpNet>() {
        return network(nets, &cidr.addr()).ok_or(Unresolved::Unknown);
    }
    if !target.contains("://") {
        if let Some((h, u)) = target.split_once('/') {
            return host_unit(facts, h, u);
        }
    }
    if facts.hosts.contains_key(target) {
        return Ok(Endpoint::Host(target.into()));
    }
    if let Some(e) = unique_unit(facts, target)? {
        return Ok(e);
    }
    if let Some(n) = nets.iter().find(|n| n.name.as_deref() == Some(target)) {
        return Ok(Endpoint::Network(n.id()));
    }
    if target == LAN {
        return Err(Unresolved::Lan);
    }
    let Some(t) = Target::parse(target) else {
        return Err(Unresolved::Unknown);
    };
    if t.is_loopback() {
        let port = t.port.map(|p| p.to_string()).unwrap_or_default();
        return unit_on_port(facts, from, t.port).ok_or(Unresolved::NoPort(from.into(), port));
    }
    let ip = match t.host {
        Host::Ipv4(ip) => Some(IpAddr::V4(ip)),
        Host::Ipv6(ip) => Some(IpAddr::V6(ip)),
        Host::Domain(_) => None,
    };
    if let Some(ip) = ip {
        return match owner(nets, &ip) {
            Some(m) => Ok(on_host(facts, &m.host, t.port)),
            None => network(nets, &ip).ok_or(Unresolved::Unknown),
        };
    }
    let name = t.host.to_string();
    if let Some(e) = book.lookup(&name, t.port)? {
        return Ok(e);
    }
    let first = name.split('.').next().unwrap_or(&name);
    if facts.hosts.contains_key(first) {
        return Ok(on_host(facts, first, t.port));
    }
    Err(Unresolved::Unknown)
}

fn network(nets: &[Network], ip: &IpAddr) -> Option<Endpoint> {
    let n = nets.iter().find(|n| n.contains(ip))?;
    Some(Endpoint::Network(n.id()))
}

fn on_host(facts: &Facts, host: &str, port: Option<u32>) -> Endpoint {
    unit_on_port(facts, host, port).unwrap_or(Endpoint::Host(host.into()))
}

fn host_unit(facts: &Facts, host: &str, unit: &str) -> Result<Endpoint, Unresolved> {
    let Some(h) = facts.hosts.get(host) else {
        return Err(Unresolved::Host(host.into()));
    };
    if h.topology().units.contains_key(unit) || h.units().any(|u| u.name == unit) {
        Ok(Endpoint::Unit(host.into(), unit.into()))
    } else {
        let (unit, host) = (unit.into(), host.into());
        Err(Unresolved::NotEnabled { unit, host })
    }
}

fn unique_unit(facts: &Facts, unit: &str) -> Result<Option<Endpoint>, Unresolved> {
    let declared: Vec<&String> = facts
        .hosts
        .iter()
        .filter(|(_, h)| h.topology().units.contains_key(unit))
        .map(|(host, _)| host)
        .collect();
    let hosts = if declared.is_empty() {
        facts
            .hosts
            .iter()
            .filter(|(_, h)| h.units().any(|u| u.name == unit))
            .map(|(host, _)| host)
            .collect()
    } else {
        declared
    };
    match hosts.as_slice() {
        [] => Ok(None),
        [h] => Ok(Some(Endpoint::Unit((*h).clone(), unit.into()))),
        _ => Err(Unresolved::AmbiguousUnit(
            unit.into(),
            hosts.iter().join(", "),
        )),
    }
}

fn unit_on_port(facts: &Facts, host: &str, port: Option<u32>) -> Option<Endpoint> {
    let port = port?;
    let topo = facts.hosts.get(host)?.topology();
    topo.units
        .iter()
        .find(|(_, u)| u.ports.contains(&port) || u.expose.iter().any(|e| e.port == port))
        .map(|(unit, _)| Endpoint::Unit(host.into(), unit.clone()))
}
