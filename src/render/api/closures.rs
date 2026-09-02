//! `closures.json` — per-host system closure sizes.
//!
//! Per *package*, never per store path, and that is a necessity rather than a
//! preference: Nix records a reference for every store path appearing in a
//! build output, so naming one here would make the docs derivation retain the
//! entire closure it describes — and `services.nixdiag.serve` would drag that
//! into the serving host's own system. Since this document cannot name a
//! path, it cannot report one.
//!
//! Unlike the treemap it is untruncated: 24 tiles is a limit on what can be
//! drawn legibly, not on what a reader may want.

use crate::api::{self, Meta};
use crate::closures::Closures;
use crate::facts::Facts;

pub(super) fn build(meta: Meta, facts: &Facts, closures: &Closures) -> api::Closures {
    let (dedup_paths, dedup_bytes) = closures.deduped();
    let shared = closures.shared();
    let fleet = api::Fleet {
        measured_hosts: closures.hosts.len(),
        shared_bytes: shared.iter().map(|(_, s)| s).sum(),
        shared_paths: shared.len(),
        deduplicated_bytes: dedup_bytes,
        deduplicated_paths: dedup_paths,
        naive_sum_bytes: closures.naive_sum(),
    };

    // Every NixOS host keeps a row. A silent omission would read as "this is
    // the whole fleet", when in fact a host can be unmeasured for two very
    // different reasons: darwin, or serving these docs.
    let hosts = facts
        .hosts
        .iter()
        .filter(|(_, h)| h.as_nixos().is_some())
        .map(|(name, _)| match closures.hosts.get(name) {
            None => api::HostClosure {
                name: name.clone(),
                measured: false,
                total_bytes: None,
                paths: None,
                split: None,
                packages: Vec::new(),
            },
            Some(h) => {
                let s = closures.split(name);
                api::HostClosure {
                    name: name.clone(),
                    measured: true,
                    total_bytes: Some(h.total()),
                    paths: Some(h.len()),
                    split: Some(api::Split {
                        shared_bytes: s.shared,
                        partial_bytes: s.partial,
                        unique_bytes: s.unique,
                    }),
                    packages: closures
                        .package_shares(name)
                        .into_iter()
                        .map(|(name, bytes, holders)| api::Package {
                            name,
                            bytes,
                            holders,
                        })
                        .collect(),
                }
            }
        })
        .collect();

    api::Closures { meta, fleet, hosts }
}
