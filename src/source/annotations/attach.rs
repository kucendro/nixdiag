use super::model::Endpoint;
use super::scan::{Raw, RawAttach};
use crate::facts::Facts;
use crate::source::imports::{build_import_graph, host_entry_modules};
use crate::source::repo::Repo;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

pub(super) struct Ctx {
    pub(super) host_order: Vec<String>,
    pub(super) unit_hosts: BTreeMap<String, Vec<String>>,
    file_units: BTreeMap<String, Vec<(String, String)>>,
    reach: HashMap<String, HashSet<String>>,
    entry_of: HashMap<String, String>,
}

impl Ctx {
    pub(super) fn build(facts: &Facts, repo: &Repo) -> Self {
        let host_order: Vec<String> = facts.hosts.keys().cloned().collect();
        let mut unit_hosts: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut file_units: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
        for (host, f) in &facts.hosts {
            for unit in f.units() {
                unit_hosts
                    .entry(unit.name.clone())
                    .or_default()
                    .push(host.clone());
                for rel in repo.repo_files(&unit.files) {
                    let e = file_units.entry(rel).or_default();
                    let pair = (host.clone(), unit.name.clone());
                    if !e.contains(&pair) {
                        e.push(pair);
                    }
                }
            }
        }
        let flake_text = std::fs::read_to_string(repo.root.join("flake.nix")).unwrap_or_default();
        let mut reach = HashMap::new();
        let mut entry_of = HashMap::new();
        let rel = |p: &Path| -> String {
            p.strip_prefix(&repo.root)
                .unwrap_or(p)
                .to_string_lossy()
                .replace('\\', "/")
        };
        for host in facts.hosts.keys() {
            let entries = host_entry_modules(host, &flake_text, repo);
            for e in &entries {
                entry_of.insert(rel(e), host.clone());
            }
            let (nodes, _) = build_import_graph(&entries, repo);
            reach.insert(host.clone(), nodes);
        }
        Ctx {
            host_order,
            unit_hosts,
            file_units,
            reach,
            entry_of,
        }
    }

    fn hosts_reaching(&self, file: &str) -> Vec<String> {
        self.host_order
            .iter()
            .filter(|h| self.reach.get(*h).is_some_and(|r| r.contains(file)))
            .cloned()
            .collect()
    }

    pub(super) fn attach(&self, raw: &Raw) -> Result<Vec<Endpoint>, String> {
        match &raw.attach {
            RawAttach::Unit(u) => {
                let via = self.hosts_reaching(&raw.file);
                let hosts = match self.unit_hosts.get(u) {
                    Some(hosts) => {
                        let narrowed: Vec<String> =
                            hosts.iter().filter(|h| via.contains(h)).cloned().collect();
                        if narrowed.is_empty() {
                            hosts.clone()
                        } else {
                            narrowed
                        }
                    }
                    None if !via.is_empty() => via,
                    None => {
                        return Err(format!(
                            "`{u}` is not enabled on any host (and no host's import graph reaches this file)"
                        ))
                    }
                };
                let mut hosts = hosts;
                hosts.sort_by_key(|h| self.host_order.iter().position(|x| x == h));
                Ok(hosts
                    .into_iter()
                    .map(|h| Endpoint::Unit(h, u.clone()))
                    .collect())
            }
            RawAttach::Declared(name) => {
                if let Some((host, unit)) = name.split_once('/') {
                    if !self.host_order.iter().any(|h| h == host) {
                        return Err(format!("unknown host `{host}` in `unit {name}`"));
                    }
                    return Ok(vec![Endpoint::Unit(host.to_string(), unit.to_string())]);
                }
                let via = self.hosts_reaching(&raw.file);
                if via.is_empty() {
                    return Err(format!(
                        "cannot place declared unit `{name}`: no host's import graph reaches this file"
                    ));
                }
                Ok(via
                    .into_iter()
                    .map(|h| Endpoint::Unit(h, name.clone()))
                    .collect())
            }
            RawAttach::File => {
                if let Some(host) = self.entry_of.get(&raw.file) {
                    return Ok(vec![Endpoint::Host(host.clone())]);
                }
                match self.file_units.get(&raw.file) {
                    Some(pairs) => Ok(pairs
                        .iter()
                        .map(|(h, u)| Endpoint::Unit(h.clone(), u.clone()))
                        .collect()),
                    None => Err(
                        "annotation attaches to nothing: not above a services./programs. \
                         binding, not a host entry module, and this file defines no \
                         service or program"
                            .into(),
                    ),
                }
            }
        }
    }
}
