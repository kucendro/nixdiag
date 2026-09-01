//! The Closures page: what each host's system actually weighs, and how much
//! of that weight the fleet shares.
//!
//! Opt-in and mode B only — nar sizes exist only for realised paths, so this
//! page is present exactly when `mkDocs { closures = …; }` supplied the data.

use super::super::out::{Out, MD_MARKER};
use crate::closures::{Closures, HostClosure};
use crate::facts::Facts;
use crate::util::{human_count, human_size, store_name};
use anyhow::Result;
use std::path::Path;

/// Biggest contributors listed per host. Enough to see what dominates without
/// turning the page into a store dump.
const TOP_PATHS: usize = 10;

pub(super) fn page_closures(
    out: &mut Out,
    src: &Path,
    facts: &Facts,
    closures: &Closures,
) -> Result<()> {
    // Every NixOS host gets a row, measured or not. `closures` accepts an
    // opt-in host list, so dropping the unselected ones would leave the page
    // reading as though it covered the whole fleet. Canonical host order comes
    // from the facts, so this page agrees with every other one.
    let hosts: Vec<(&str, Option<&HostClosure>)> = facts
        .hosts
        .iter()
        .filter(|(_, h)| h.as_nixos().is_some())
        .map(|(n, _)| (n.as_str(), closures.hosts.get(n.as_str())))
        .collect();

    let mut o: Vec<String> = vec![
        MD_MARKER.into(),
        "".into(),
        "# Closures".into(),
        "".into(),
        "What each host's system closure weighs, measured from the realised \
         store paths. NixOS hosts only — a darwin system cannot be built from \
         Linux, so those hosts are absent here. A host shown as — was not \
         selected for measurement; note that a host serving these docs cannot \
         measure itself, as the docs would then depend on a system that \
         contains them."
            .into(),
        "".into(),
    ];
    o.extend(summary_rows(closures, &hosts));
    o.push("".into());

    // With one measured host every path is trivially both shared and unique,
    // so the comparison says nothing.
    let measured = hosts.iter().filter(|(_, c)| c.is_some()).count();
    if measured > 1 {
        let shared = closures.shared();
        let shared_size: u64 = shared.iter().map(|(_, s)| s).sum();
        let (dedup_n, dedup_size) = closures.deduped();
        let naive = closures.naive_sum();

        o.push("## Fleet".into());
        o.push("".into());
        o.push("| | |".into());
        o.push("|---|---|".into());
        o.push(format!(
            "| Shared by every host | {} ({} paths) |",
            human_size(shared_size),
            human_count(shared.len())
        ));
        o.push(format!(
            "| Fleet total, deduplicated | {} ({} paths) |",
            human_size(dedup_size),
            human_count(dedup_n)
        ));
        o.push(format!(
            "| Sum of per-host closures | {} |",
            human_size(naive)
        ));
        o.push(format!(
            "| Saved by sharing | {} |",
            human_size(naive.saturating_sub(dedup_size))
        ));
        o.push("".into());
    }

    for (host, closure) in &hosts {
        // An unmeasured host has nothing to contribute; an empty heading would
        // only repeat what the summary table already said.
        let Some(h) = closure else { continue };
        o.push(format!("## {host} — largest contributors"));
        o.push("".into());
        o.push("| Package | Size |".into());
        o.push("|---|---|".into());
        for p in h.largest(TOP_PATHS) {
            // Name only, never the full path — see util::store_name.
            o.push(format!(
                "| `{}` | {} |",
                store_name(&p.path),
                human_size(p.nar_size)
            ));
        }
        if h.paths.is_empty() {
            o.push("| — | — |".into());
        }
        o.push("".into());
    }

    out.write_auto(&src.join("closures.md"), &o.join("\n"))
}

/// The summary table. Split out so the unmeasured-host case is testable
/// without building a whole `Facts`.
fn summary_rows(closures: &Closures, hosts: &[(&str, Option<&HostClosure>)]) -> Vec<String> {
    let mut o = vec![
        "| Host | Closure | Paths | Unique |".to_string(),
        "|---|---|---|---|".to_string(),
    ];
    if hosts.is_empty() {
        o.push("| — | — | — | — |".into());
        return o;
    }
    for (host, closure) in hosts {
        match closure {
            Some(h) => {
                let unique: u64 = closures.unique(host).iter().map(|(_, s)| s).sum();
                o.push(format!(
                    "| `{host}` | {} | {} | {} |",
                    human_size(h.total()),
                    human_count(h.len()),
                    human_size(unique),
                ));
            }
            None => o.push(format!("| `{host}` | — | — | — |")),
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::closures::ClosurePath;
    use indexmap::IndexMap;

    fn closures() -> Closures {
        let mut hosts = IndexMap::new();
        hosts.insert(
            "nas".to_string(),
            HostClosure {
                paths: vec![ClosurePath {
                    path: "/nix/store/0000000000000000000000000000000a-bash-5.2".into(),
                    nar_size: 1024,
                }],
            },
        );
        Closures { schema: 1, hosts }
    }

    #[test]
    fn unselected_hosts_still_get_a_row() {
        let c = closures();
        let nas = c.hosts.get("nas");
        let rows = summary_rows(&c, &[("nas", nas), ("edge", None)]);
        assert!(
            rows.iter().any(|r| r.starts_with("| `nas` | 1.0 KiB")),
            "{rows:?}"
        );
        assert!(
            rows.contains(&"| `edge` | — | — | — |".to_string()),
            "{rows:?}"
        );
    }

    #[test]
    fn no_nixos_hosts_at_all_renders_a_placeholder() {
        let rows = summary_rows(&closures(), &[]);
        assert_eq!(rows.last().unwrap(), "| — | — | — | — |");
    }
}
