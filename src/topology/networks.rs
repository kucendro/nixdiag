use crate::facts::{Declared, Facts, Scope};
use crate::text::messages::Fail;
use anyhow::{Error, Result};
use indexmap::IndexMap;
use ipnet::IpNet;
use std::collections::BTreeSet;
use std::net::IpAddr;

#[derive(Debug)]
pub struct Member {
    pub host: String,
    pub interface: String,
    pub address: Option<IpAddr>,
    pub routes: Vec<IpNet>,
}

#[derive(Debug)]
pub struct Network {
    pub name: Option<String>,
    pub cidrs: Vec<IpNet>,
    pub kind: Option<Scope>,
    pub server: Option<String>,
    pub members: Vec<Member>,
    pub gateways: BTreeSet<IpAddr>,
}

impl Network {
    fn new(name: Option<String>, cidrs: Vec<IpNet>, kind: Option<Scope>) -> Self {
        Network {
            name,
            cidrs,
            kind,
            server: None,
            members: Vec::new(),
            gateways: BTreeSet::new(),
        }
    }

    pub fn id(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| self.cidrs[0].to_string())
    }

    fn covers(&self, prefix: &IpNet) -> Option<u8> {
        let within = self.cidrs.iter().filter(|c| c.contains(prefix));
        within.map(IpNet::prefix_len).max()
    }

    fn overlaps(&self, other: &Network) -> bool {
        self.cidrs
            .iter()
            .any(|a| other.cidrs.iter().any(|b| a.contains(b) || b.contains(a)))
    }
}

pub fn widest(nets: &[Network], ids: &[String]) -> Option<Scope> {
    let rank = |s: &Scope| match s {
        Scope::Mesh => 0,
        Scope::Lan => 1,
        Scope::Public => 2,
    };
    let kinds = nets.iter().filter(|n| ids.contains(&n.id()));
    kinds.filter_map(|n| n.kind).max_by_key(rank)
}

fn at(nets: &[Network], prefix: &IpNet) -> Option<usize> {
    let covering = nets.iter().enumerate();
    let covering = covering.filter_map(|(i, n)| Some((n.covers(prefix)?, i)));
    covering.max_by_key(|(len, _)| *len).map(|(_, i)| i)
}

pub fn longest<'a>(nets: &'a [Network], prefix: &IpNet) -> Option<&'a Network> {
    at(nets, prefix).map(|i| &nets[i])
}

pub fn owner<'a>(nets: &'a [Network], ip: &IpAddr) -> Option<&'a Member> {
    let members = nets.iter().flat_map(|n| &n.members);
    members.into_iter().find(|m| m.address.as_ref() == Some(ip))
}

fn address(host: &str, value: &str) -> Error {
    Fail::Address {
        host: host.into(),
        value: value.into(),
    }
    .into()
}

fn declared(facts: &Facts) -> Result<Vec<Network>> {
    let mut seen: IndexMap<&str, (&str, &Declared)> = IndexMap::new();
    for (host, h) in &facts.hosts {
        for (name, d) in &h.topology().networks {
            match seen.get(name.as_str()) {
                Some((first, prev)) if *prev != d => {
                    return Err(Fail::NetworkConflict {
                        name: name.clone(),
                        first: first.to_string(),
                        second: host.clone(),
                    }
                    .into())
                }
                Some(_) => {}
                None => {
                    seen.insert(name, (host, d));
                }
            }
        }
    }
    let mut nets = Vec::new();
    for (name, (host, d)) in seen {
        let cidrs = d.cidrs.iter().map(|c| {
            c.parse::<IpNet>()
                .map(|n| n.trunc())
                .map_err(|_| address(host, c))
        });
        let mut net = Network::new(Some(name.into()), cidrs.collect::<Result<_>>()?, d.kind);
        net.server.clone_from(&d.server);
        if let Some(other) = nets.iter().find(|o: &&Network| o.overlaps(&net)) {
            return Err(Fail::NetworkOverlap(other.id(), net.id()).into());
        }
        nets.push(net);
    }
    Ok(nets)
}

pub fn build(facts: &Facts) -> Result<Vec<Network>> {
    let mut nets = declared(facts)?;
    for (host, h) in &facts.hosts {
        let Some(n) = h.as_nixos() else { continue };
        for (name, i) in &n.network.interfaces {
            let routes = i.routes.iter().map(|r| {
                r.parse::<IpNet>()
                    .map(|n| n.trunc())
                    .map_err(|_| address(host, r))
            });
            let routes: Vec<IpNet> = routes.collect::<Result<_>>()?;
            let member = |address| Member {
                host: host.clone(),
                interface: name.clone(),
                address,
                routes: routes.clone(),
            };
            for a in &i.addresses {
                let Some(scope) = a.scope else { continue };
                let net: IpNet = a.cidr.parse().map_err(|_| address(host, &a.cidr))?;
                let slot = match at(&nets, &net.addr().into()) {
                    Some(slot) => slot,
                    None if net.prefix_len() == net.max_prefix_len() => continue,
                    None => {
                        nets.push(Network::new(None, vec![net.trunc()], Some(scope)));
                        nets.len() - 1
                    }
                };
                nets[slot].kind.get_or_insert(scope);
                nets[slot].members.push(member(Some(net.addr())));
            }
            let by_server = i.server.as_ref().filter(|_| i.addresses.is_empty());
            if let Some(n) =
                by_server.and_then(|s| nets.iter_mut().find(|n| n.server.as_ref() == Some(s)))
            {
                n.members.push(member(None));
            }
        }
        for g in &n.network.gateways {
            let ip: IpAddr = g.address.parse().map_err(|_| address(host, &g.address))?;
            if let Some(i) = at(&nets, &ip.into()) {
                nets[i].gateways.insert(ip);
            }
        }
    }
    let units = facts
        .hosts
        .values()
        .flat_map(|h| h.topology().units.values());
    let targets = units
        .flat_map(|u| &u.connections)
        .filter_map(|c| c.to.parse::<IpNet>().ok());
    let members = nets.iter().flat_map(|n| &n.members);
    let routes: Vec<IpNet> = members.flat_map(|m| m.routes.iter().copied()).collect();
    for net in targets.chain(routes).filter(|n| n.prefix_len() > 0) {
        if at(&nets, &net).is_none() {
            nets.push(Network::new(None, vec![net.trunc()], None));
        }
    }
    Ok(nets)
}

#[cfg(test)]
mod tests;
