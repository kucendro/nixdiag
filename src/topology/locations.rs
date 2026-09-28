use super::networks::Network;
use crate::facts::{Facts, Scope};
use indexmap::{IndexMap, IndexSet};

pub fn build(facts: &Facts, nets: &[Network]) -> IndexMap<String, String> {
    let mut groups: Vec<IndexSet<&str>> = facts
        .hosts
        .keys()
        .map(|h| IndexSet::from([h.as_str()]))
        .collect();
    for n in nets {
        if !matches!(n.kind, Some(Scope::Lan | Scope::Public)) {
            continue;
        }
        let hosts: IndexSet<&str> = n.members.iter().map(|m| m.host.as_str()).collect();
        let (joined, rest): (Vec<_>, Vec<_>) =
            groups.into_iter().partition(|g| !g.is_disjoint(&hosts));
        groups = rest;
        groups.push(joined.into_iter().flatten().collect());
    }
    let own = |h: &str| facts.hosts.get(h)?.topology().location.as_deref();
    let mut out = IndexMap::new();
    for host in facts.hosts.keys() {
        let group = groups.iter().find(|g| g.contains(host.as_str()));
        let named: IndexSet<&str> = group.into_iter().flatten().filter_map(|m| own(m)).collect();
        let shared = named.first().filter(|_| named.len() == 1);
        if let Some(name) = own(host).or(shared.copied()) {
            out.insert(host.clone(), name.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests;
