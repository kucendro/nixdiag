//! `services.json` — every service this repo configures, which hosts enable
//! it, and the files that define it.
//!
//! "Configures" means the same thing here as on the Services page: a unit
//! with at least one defining file inside this repo. The projection also sees
//! units enabled by nixpkgs itself, and listing those would bury the handful
//! the repo actually owns.

use crate::api::{self, Meta};
use crate::facts::{Facts, Host};
use crate::source::repo::Repo;
use std::collections::{BTreeMap, BTreeSet};

/// One host's repo-configured units, as (name, repo-relative files), split
/// into services and programs.
///
/// Shared with `hosts.rs` so the two documents cannot disagree about which
/// units count as this repo's.
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

/// (kind, name) -> (hosts, files). Kind is part of the key so a service and
/// a program of the same name stay distinct rather than merging.
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
