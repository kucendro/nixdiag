use crate::api::{self, Meta};
use crate::facts::{Facts, Host};
use crate::source::repo::Repo;
use std::collections::{BTreeMap, BTreeSet};

type Units = (Vec<(String, Vec<String>)>, Vec<(String, Vec<String>)>);

pub(super) fn repo_units(host: &Host, repo: &Repo) -> Units {
    let (services, programs) = match host {
        Host::Nixos(n) => (&n.services, &n.programs),
        Host::Darwin(d) => (&d.services, &d.programs),
    };
    let pick = |units: &Vec<crate::facts::EnabledUnit>| -> Vec<(String, Vec<String>)> {
        units
            .iter()
            .filter_map(|u| {
                let files = repo.repo_files(&u.files);
                (!files.is_empty()).then(|| (u.name.clone(), files))
            })
            .collect()
    };
    (pick(services), pick(programs))
}

type Index = BTreeMap<(&'static str, String), (BTreeSet<String>, BTreeSet<String>)>;

pub(super) fn build(meta: Meta, facts: &Facts, repo: &Repo) -> api::Services {
    let mut index: Index = BTreeMap::new();
    for (host, f) in &facts.hosts {
        let (services, programs) = repo_units(f, repo);
        for (kind, units) in [("service", services), ("program", programs)] {
            for (name, files) in units {
                let e = index.entry((kind, name)).or_default();
                e.0.insert(host.clone());
                e.1.extend(files);
            }
        }
    }
    api::Services {
        meta,
        services: index
            .into_iter()
            .map(|((kind, name), (hosts, files))| api::ServiceEntry {
                name,
                kind,
                hosts: hosts.into_iter().collect(),
                files: files.into_iter().collect(),
            })
            .collect(),
    }
}
