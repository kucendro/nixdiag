use super::Endpoint;
use crate::facts::Facts;
use crate::text::{fill, messages as m};
use std::collections::BTreeMap;

pub struct Book(BTreeMap<String, Vec<(Endpoint, Option<u32>)>>);

impl Book {
    pub fn new(facts: &Facts) -> Self {
        let mut book: BTreeMap<String, Vec<(Endpoint, Option<u32>)>> = BTreeMap::new();
        let mut add = |name: &str, node: &Endpoint, port: Option<u32>| {
            book.entry(name.to_string())
                .or_default()
                .push((node.clone(), port));
        };
        for (host, h) in &facts.hosts {
            let topo = h.topology();
            let node = Endpoint::Host(host.clone());
            for n in &topo.names {
                add(n, &node, None);
            }
            for e in &topo.expose {
                if let Some(n) = &e.name {
                    add(n, &node, Some(e.port));
                }
            }
            for (unit, u) in &topo.units {
                let node = Endpoint::Unit(host.clone(), unit.clone());
                for n in &u.names {
                    add(n, &node, None);
                }
                for e in &u.expose {
                    if let Some(n) = &e.name {
                        add(n, &node, Some(e.port));
                    }
                }
                for c in &u.connections {
                    if let Some(n) = &c.name {
                        add(n, &node, c.port);
                    }
                }
            }
        }
        Book(book)
    }

    fn lookup(&self, name: &str, port: Option<u32>) -> Result<Option<Endpoint>, String> {
        let Some(entries) = self.0.get(name) else {
            return Ok(None);
        };
        let mut nodes = unique(entries.iter().filter(|(_, p)| port.is_some() && *p == port));
        if nodes.is_empty() {
            nodes = unique(entries.iter());
        }
        match nodes.as_slice() {
            [n] => Ok(Some(n.clone())),
            _ => Err(fill(m::AMBIGUOUS_NAME, &[("name", name)])),
        }
    }
}

fn unique<'a>(entries: impl Iterator<Item = &'a (Endpoint, Option<u32>)>) -> Vec<Endpoint> {
    let mut v: Vec<Endpoint> = Vec::new();
    for (n, _) in entries {
        if !v.contains(n) {
            v.push(n.clone());
        }
    }
    v
}

pub fn target(facts: &Facts, book: &Book, from: &str, target: &str) -> Result<Endpoint, String> {
    if target == "internet" {
        return Ok(Endpoint::Internet);
    }
    if target == "lan" {
        return Ok(Endpoint::Lan);
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
    let (name, port) = split_url(target);
    if is_loopback(name) {
        let port_text = port.map(|p| p.to_string()).unwrap_or_default();
        return unit_on_port(facts, from, port)
            .ok_or_else(|| fill(m::NO_PORT, &[("host", from), ("port", &port_text)]));
    }
    if let Some(e) = book.lookup(name, port)? {
        return Ok(e);
    }
    let first = name.split('.').next().unwrap_or(name);
    if facts.hosts.contains_key(first) {
        return Ok(unit_on_port(facts, first, port).unwrap_or(Endpoint::Host(first.into())));
    }
    Err(m::UNKNOWN_TARGET.into())
}

fn host_unit(facts: &Facts, host: &str, unit: &str) -> Result<Endpoint, String> {
    let Some(h) = facts.hosts.get(host) else {
        return Err(fill(m::UNKNOWN_HOST, &[("host", host)]));
    };
    if h.topology().units.contains_key(unit) || h.units().any(|u| u.name == unit) {
        Ok(Endpoint::Unit(host.into(), unit.into()))
    } else {
        Err(fill(m::NOT_ENABLED, &[("unit", unit), ("host", host)]))
    }
}

fn unique_unit(facts: &Facts, unit: &str) -> Result<Option<Endpoint>, String> {
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
        _ => {
            let list = hosts.iter().map(|h| h.as_str()).collect::<Vec<_>>();
            Err(fill(
                m::AMBIGUOUS_UNIT,
                &[("unit", unit), ("hosts", &list.join(", "))],
            ))
        }
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

fn split_url(target: &str) -> (&str, Option<u32>) {
    let (scheme, rest) = match target.split_once("://") {
        Some((s, r)) => (Some(s), r),
        None => (None, target),
    };
    let authority = rest.split('/').next().unwrap_or(rest);
    let (host, port) = match authority.strip_prefix('[').and_then(|a| a.split_once(']')) {
        Some((h, p)) => (h, p.strip_prefix(':')),
        None => match authority.rsplit_once(':') {
            Some((h, p)) => (h, Some(p)),
            None => (authority, None),
        },
    };
    let default = match scheme {
        Some("https") => Some(443),
        Some("http") => Some(80),
        _ => None,
    };
    (host, port.and_then(|p| p.parse().ok()).or(default))
}

fn is_loopback(host: &str) -> bool {
    host.starts_with("127.") || host == "localhost" || host == "::1"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_split_into_host_and_port_with_scheme_defaults() {
        assert_eq!(
            split_url("https://hs.ts.example"),
            ("hs.ts.example", Some(443))
        );
        assert_eq!(
            split_url("http://luna.ts.example:3000/api"),
            ("luna.ts.example", Some(3000))
        );
        assert_eq!(split_url("127.0.0.1:8080"), ("127.0.0.1", Some(8080)));
        assert_eq!(split_url("http://[::1]:9090"), ("::1", Some(9090)));
        assert_eq!(split_url("db.example"), ("db.example", None));
    }
}
