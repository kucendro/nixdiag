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
