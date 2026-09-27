use super::{codes, repo_services, table, Page, Wiki};
use crate::conf::files::page;
use crate::text::wiki::services as t;
use anyhow::Result;
use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Services;

impl Page for Services {
    fn file(&self) -> &'static str {
        page::SERVICES
    }

    fn title(&self) -> &'static str {
        t::TITLE
    }

    fn body(&self, w: &Wiki) -> Result<Option<Vec<String>>> {
        Ok(Some(body(w)))
    }
}

fn body(w: &Wiki) -> Vec<String> {
    let mut index: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
    for (host, f) in &w.facts.hosts {
        let Some(n) = f.as_nixos() else { continue };
        for (name, files) in repo_services(&n.base, w.repo) {
            let e = index.entry(name).or_default();
            e.0.insert(host.clone());
            e.1.extend(files);
        }
    }
    let rows = index
        .iter()
        .map(|(name, (hosts, files))| [t::name(name), hosts.iter().join(", "), codes(files, " ")]);
    let mut o = vec![table(t::HEAD, rows)];
    let mut described: BTreeMap<&str, &str> = BTreeMap::new();
    for f in w.facts.hosts.values() {
        for (name, u) in &f.topology().units {
            if let Some(d) = &u.description {
                described.entry(name).or_insert(d);
            }
        }
    }
    o.extend(described.iter().map(|(name, d)| t::unit(name, d)));
    o
}
