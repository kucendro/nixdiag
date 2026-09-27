use super::{codes, repo_services, table, Wiki};
use crate::conf::files::page;
use crate::text::wiki::services as t;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn page_services(w: &Wiki) -> Result<()> {
    let mut index: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
    for (host, f) in &w.facts.hosts {
        let Some(n) = f.as_nixos() else { continue };
        for (name, files) in repo_services(&n.base, w.repo) {
            let e = index.entry(name).or_default();
            e.0.insert(host.clone());
            e.1.extend(files);
        }
    }
    let rows = index.iter().map(|(name, (hosts, files))| {
        let hosts = hosts.iter().cloned().collect::<Vec<_>>().join(", ");
        [t::name(name), hosts, codes(files, " ")]
    });
    let mut o = vec![t::TITLE.to_string(), table(t::HEAD, rows)];
    let mut described: BTreeMap<&str, &str> = BTreeMap::new();
    for f in w.facts.hosts.values() {
        for (name, u) in &f.topology().units {
            if let Some(d) = &u.description {
                described.entry(name).or_insert(d);
            }
        }
    }
    o.extend(described.iter().map(|(name, d)| t::unit(name, d)));
    w.page(page::SERVICES, &o)
}
