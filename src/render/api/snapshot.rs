//! `snapshot.json` — the small document a trend line is built from.
//!
//! A few hundred bytes plus one number per host, deliberately not a copy of
//! everything else: history means fetching many of these, so size is the
//! design constraint. It is also the only document that is not a pure
//! function of the repo, since it carries the revision — which is why it is
//! written `Volatile` and stays out of `nixdiag check`.

use super::ApiData;
use crate::api::{self, Meta};
use crate::facts::Host;

pub(super) fn build(meta: Meta, revision: Option<api::Revision>, d: &ApiData) -> api::Snapshot {
    let nixos = d
        .facts
        .hosts
        .values()
        .filter(|h| matches!(h, Host::Nixos(_)))
        .count();

    let mut services = 0;
    let mut programs = 0;
    let (mut tcp, mut udp) = (0, 0);
    let mut packages = 0u64;
    for h in d.facts.hosts.values() {
        let (s, p) = super::services::repo_units(h, d.repo);
        services += s.len();
        programs += p.len();
        if let Some(n) = h.as_nixos() {
            tcp += n.tcp.len();
            udp += n.udp.len();
            packages += n.pkg_count;
        }
    }

    let exposes: usize = d
        .model
        .hosts
        .values()
        .chain(d.model.units.values())
        .map(|i| i.exposes.len())
        .sum();

    let inputs = d.lock.map(|lock| {
        let dups = lock.duplicates();
        let span = lock.date_span();
        api::InputTotals {
            total: lock.inputs().len(),
            direct: lock.root_inputs().len(),
            diamonds: dups.iter().filter(|d| d.is_diamond()).count(),
            redundant: dups.iter().filter(|d| !d.is_diamond()).count(),
            oldest: span.map(|(lo, _)| lo),
            newest: span.map(|(_, hi)| hi),
            span_days: span.map(|(lo, hi)| (hi - lo) / 86_400),
        }
    });

    let closures = d.closures.map(|c| api::ClosureTotals {
        measured: c.hosts.len(),
        deduplicated_bytes: c.deduped().1,
        naive_sum_bytes: c.naive_sum(),
        hosts: c
            .hosts
            .iter()
            .map(|(name, h)| (name.clone(), h.total()))
            .collect(),
    });

    api::Snapshot {
        meta,
        revision,
        totals: api::Totals {
            hosts: d.facts.hosts.len(),
            nixos_hosts: nixos,
            darwin_hosts: d.facts.hosts.len() - nixos,
            services,
            programs,
            ports: api::PortTotals { tcp, udp },
            packages,
            annotations: api::AnnotationTotals {
                statements: d.model.total,
                nodes: d.model.hosts.len() + d.model.units.len(),
                edges: d.model.edges.len(),
                endpoints: exposes + d.model.named.len(),
            },
            inputs,
            closures,
        },
    }
}
