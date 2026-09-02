//! `hosts.json` — one entry per configuration, with whatever the projection
//! saw plus any host-level annotation.

use super::services::repo_units;
use crate::api::{self, Meta};
use crate::facts::{Facts, Host};
use crate::source::annotations::Model;
use crate::source::repo::Repo;

pub(super) fn build(meta: Meta, facts: &Facts, repo: &Repo, model: &Model) -> api::Hosts {
    let hosts = facts
        .hosts
        .iter()
        .map(|(name, h)| {
            let info = model.hosts.get(name);
            let (services, programs) = repo_units(h, repo);
            // A darwin host has no platform, stateVersion, package count or
            // firewall in the projection. Emitted as null rather than omitted,
            // so a reader can index every key on every entry.
            let n = h.as_nixos();
            api::HostEntry {
                name: name.clone(),
                kind: match h {
                    Host::Nixos(_) => "nixos",
                    Host::Darwin(_) => "darwin",
                },
                platform: n.map(|n| n.platform.clone()),
                state_version: n.map(|n| n.state_version.clone()),
                packages: n.map(|n| n.pkg_count),
                users: n.map(|n| n.users.clone()).unwrap_or_default(),
                ports: api::Ports {
                    tcp: n.map(|n| n.tcp.clone()).unwrap_or_default(),
                    udp: n.map(|n| n.udp.clone()).unwrap_or_default(),
                },
                role: info.and_then(|i| i.role.clone()),
                scope: info.and_then(|i| i.scope).map(|s| s.label().to_string()),
                services: services.into_iter().map(|(n, _)| n).collect(),
                programs: programs.into_iter().map(|(n, _)| n).collect(),
            }
        })
        .collect();
    api::Hosts { meta, hosts }
}
