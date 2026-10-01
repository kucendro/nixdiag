use super::locations;
use crate::facts::{Declared, Facts, Scope};
use crate::text::d2::networks as t;
use crate::text::messages::{Fail, Unresolved};
use anyhow::{Error, Result};
use indexmap::IndexMap;
use ipnet::IpNet;
use std::collections::BTreeSet;
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct Route {
    pub host: String,
    pub interface: String,
    pub prefix: IpNet,
    pub to: Option<String>,
}

#[derive(Debug)]
pub struct Member {
    pub host: String,
    pub interface: String,
    pub address: Option<IpAddr>,
}

#[derive(Debug)]
pub struct Network {
    pub name: Option<String>,
    pub cidrs: Vec<IpNet>,
    pub kind: Option<Scope>,
    pub location: Option<String>,
    pub server: Option<String>,
    pub members: Vec<Member>,
    pub gateways: BTreeSet<IpAddr>,
}

#[derive(Debug)]
pub struct Ambiguous;

impl From<Ambiguous> for Unresolved {
    fn from(_: Ambiguous) -> Self {
        Unresolved::AmbiguousNetwork
    }
}

impl Network {
    fn new(name: Option<String>, cidrs: Vec<IpNet>, kind: Option<Scope>) -> Self {
        Network {
            name,
            cidrs,
            kind,
            location: None,
            server: None,
            members: Vec::new(),
            gateways: BTreeSet::new(),
        }
    }

    fn at(mut self, location: Option<&str>) -> Self {
        self.location = location
            .filter(|_| self.kind != Some(Scope::Mesh))
            .map(Into::into);
        self
    }

    pub fn id(&self) -> String {
        let base = self.name.clone();
        let base = base.unwrap_or_else(|| self.cidrs[0].to_string());
        ident(&base, self.location.as_deref())
    }

    pub fn covers(&self, prefix: &IpNet) -> Option<u8> {
        let within = self.cidrs.iter().filter(|c| c.contains(prefix));
        within.map(IpNet::prefix_len).max()
    }

    fn member(&self, ip: &IpAddr) -> Option<&Member> {
        self.members.iter().find(|m| m.address.as_ref() == Some(ip))
    }

    fn overlaps(&self, other: &Network) -> bool {
        let apart = matches!(
            (&self.location, &other.location),
            (Some(a), Some(b)) if a != b
        );
        !apart
            && self
                .cidrs
                .iter()
                .any(|a| other.cidrs.iter().any(|b| a.contains(b) || b.contains(a)))
    }
}

fn ident(base: &str, location: Option<&str>) -> String {
    location.map_or(base.into(), |l| t::located(base, l))
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

pub fn by_id<'a>(nets: &'a [Network], id: &str) -> Option<&'a Network> {
    nets.iter().find(|n| n.id() == id)
}

fn hits(nets: &[Network], hit: impl Fn(&Network) -> Option<u8>) -> Vec<(u8, usize)> {
    let found = nets.iter().enumerate();
    found.filter_map(|(i, n)| Some((hit(n)?, i))).collect()
}

fn choose(
    nets: &[Network],
    found: &[(u8, usize)],
    location: Option<&str>,
) -> Result<Option<usize>, Ambiguous> {
    let there = |i: usize| nets[i].location.as_deref();
    if location.is_none() {
        let places: BTreeSet<&str> = found.iter().filter_map(|(_, i)| there(*i)).collect();
        if places.len() > 1 {
            return Err(Ambiguous);
        }
    }
    let visible = found.iter().filter(|(_, i)| match (location, there(*i)) {
        (Some(here), Some(there)) => here == there,
        _ => true,
    });
    Ok(visible.max_by_key(|(rank, _)| *rank).map(|(_, i)| *i))
}

fn local(
    nets: &[Network],
    prefix: &IpNet,
    location: Option<&str>,
) -> Result<Option<usize>, Ambiguous> {
    choose(nets, &hits(nets, |n| n.covers(prefix)), location)
}

pub fn find<'a>(
    nets: &'a [Network],
    hit: impl Fn(&Network) -> Option<u8>,
    location: Option<&str>,
) -> Result<Option<&'a Network>, Ambiguous> {
    let found = hits(nets, hit);
    let slot = match choose(nets, &found, location)? {
        Some(i) => Some(i),
        None => choose(nets, &found, None)?,
    };
    Ok(slot.map(|i| &nets[i]))
}

pub fn owner<'a>(
    nets: &'a [Network],
    ip: &IpAddr,
    location: Option<&str>,
) -> Result<Option<&'a Member>, Ambiguous> {
    let net = find(nets, |n| n.member(ip).map(|_| 0), location)?;
    Ok(net.and_then(|n| n.member(ip)))
}

fn address(host: &str, value: &str) -> Error {
    Fail::Address {
        host: host.into(),
        value: value.into(),
    }
    .into()
}

fn unlocated(host: &str, value: impl ToString) -> Error {
    Fail::Unlocated {
        host: host.into(),
        value: value.to_string(),
    }
    .into()
}

fn place(facts: &Facts, host: &str) -> Option<String> {
    facts.hosts.get(host)?.topology().location.clone()
}

fn declared(facts: &Facts) -> Result<Vec<Network>> {
    let mut seen: IndexMap<String, (&str, &Declared, Network)> = IndexMap::new();
    for (host, h) in &facts.hosts {
        for (name, d) in &h.topology().networks {
            let cidrs = d.cidrs.iter().map(|c| {
                c.parse::<IpNet>()
                    .map(|n| n.trunc())
                    .map_err(|_| address(host, c))
            });
            let net = Network::new(Some(name.clone()), cidrs.collect::<Result<_>>()?, d.kind);
            let mut net = net.at(place(facts, host).as_deref());
            net.server.clone_from(&d.server);
            let id = net.id();
            match seen.get(&id) {
                Some((first, prev, _)) if *prev != d => {
                    return Err(Fail::NetworkConflict {
                        name: id,
                        first: first.to_string(),
                        second: host.clone(),
                    }
                    .into())
                }
                Some(_) => {}
                None => {
                    seen.insert(id, (host, d, net));
                }
            }
        }
    }
    let mut nets: Vec<Network> = Vec::new();
    for (_, _, net) in seen.into_values() {
        if let Some(other) = nets.iter().find(|o| o.overlaps(&net)) {
            return Err(Fail::NetworkOverlap(other.id(), net.id()).into());
        }
        nets.push(net);
    }
    Ok(nets)
}

fn join(facts: &Facts, nets: &mut Vec<Network>, host: &str) -> Result<()> {
    let Some(n) = facts.hosts[host].as_nixos() else {
        return Ok(());
    };
    let location = place(facts, host);
    let location = location.as_deref();
    for (name, i) in &n.network.interfaces {
        let member = |address| Member {
            host: host.into(),
            interface: name.clone(),
            address,
        };
        for a in &i.addresses {
            let Some(scope) = a.scope else { continue };
            let net: IpNet = a.cidr.parse().map_err(|_| address(host, &a.cidr))?;
            let found = local(nets, &net.addr().into(), location);
            let slot = match found.map_err(|_| unlocated(host, net.addr()))? {
                Some(slot) => slot,
                None if net.prefix_len() == net.max_prefix_len() => continue,
                None => {
                    let fresh = Network::new(None, vec![net.trunc()], Some(scope));
                    nets.push(fresh.at(location));
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
        let found = local(nets, &ip.into(), location).map_err(|_| unlocated(host, ip))?;
        if let Some(i) = found {
            nets[i].gateways.insert(ip);
        }
    }
    Ok(())
}

fn routes(facts: &Facts) -> Result<Vec<Route>> {
    let mut out = Vec::new();
    for (host, h) in &facts.hosts {
        let Some(n) = h.as_nixos() else { continue };
        for (interface, i) in &n.network.interfaces {
            for r in &i.routes {
                let prefix: IpNet = r.parse().map_err(|_| address(host, r))?;
                out.push(Route {
                    host: host.clone(),
                    interface: interface.clone(),
                    prefix: prefix.trunc(),
                    to: None,
                });
            }
        }
    }
    Ok(out)
}

fn route(
    nets: &mut Vec<Network>,
    routes: &mut [Route],
    places: &IndexMap<String, String>,
) -> Result<()> {
    for r in routes.iter_mut().filter(|r| r.prefix.prefix_len() > 0) {
        let location = places.get(&r.host).map(String::as_str);
        let found = local(nets, &r.prefix, location);
        let found = found.map_err(|_| unlocated(&r.host, r.prefix))?;
        let slot = found.unwrap_or_else(|| {
            nets.push(Network::new(None, vec![r.prefix], None).at(location));
            nets.len() - 1
        });
        r.to = Some(nets[slot].id());
    }
    Ok(())
}

fn targets(facts: &Facts, nets: &mut Vec<Network>, places: &IndexMap<String, String>) {
    for (host, h) in &facts.hosts {
        let location = places.get(host).map(String::as_str);
        let units = h.topology().units.values();
        let cidrs = units.flat_map(|u| &u.connections);
        for prefix in cidrs.filter_map(|c| c.to.parse::<IpNet>().ok()) {
            let prefix = prefix.trunc();
            if prefix.prefix_len() > 0
                && matches!(find(nets, |n| n.covers(&prefix), location), Ok(None))
            {
                nets.push(Network::new(None, vec![prefix], None));
            }
        }
    }
}

pub fn build(facts: &Facts) -> Result<(Vec<Network>, Vec<Route>)> {
    let mut nets = declared(facts)?;
    let (located, unplaced): (Vec<&String>, Vec<&String>) =
        facts.hosts.keys().partition(|h| place(facts, h).is_some());
    for host in located.into_iter().chain(unplaced) {
        join(facts, &mut nets, host)?;
    }
    for n in &mut nets {
        n.members.sort_by_key(|m| facts.hosts.get_index_of(&m.host));
    }
    let places = locations::build(facts, &nets);
    let mut routes = routes(facts)?;
    route(&mut nets, &mut routes, &places)?;
    targets(facts, &mut nets, &places);
    Ok((nets, routes))
}

#[cfg(test)]
mod tests;
