mod charts;
#[cfg(test)]
mod tests;

use super::super::chart;
use super::super::d2::D2Style;
use super::super::out::{Out, MD_MARKER};
use crate::closures::{Closures, HostClosure};
use crate::facts::Facts;
use crate::util::{human_count, human_size, sanitize, store_name};
use anyhow::Result;
use charts::{bar_rows, treemap_tiles};
use std::path::Path;

const TOP_PATHS: usize = 10;

pub(super) fn page_closures(
    out: &mut Out,
    src: &Path,
    facts: &Facts,
    closures: &Closures,
    style: &D2Style,
) -> Result<()> {
    let hosts: Vec<(&str, Option<&HostClosure>)> = facts
        .hosts
        .iter()
        .filter(|(_, h)| h.as_nixos().is_some())
        .map(|(n, _)| (n.as_str(), closures.hosts.get(n.as_str())))
        .collect();

    let mut o: Vec<String> = vec![MD_MARKER.into(), "".into(), "# Closures".into(), "".into()];
    if !hosts.is_empty() {
        let svg = chart::bars(
            "System closure size by host",
            &bar_rows(closures, &hosts),
            style,
        );
        out.write_auto(&src.join("closures.svg"), &svg)?;
        o.push("![System closure size by host](./closures.svg)".into());
        o.push("".into());
    }

    o.extend(summary_rows(closures, &hosts));
    o.push("".into());

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
        let Some(h) = closure else { continue };
        o.push(format!("## {host}"));
        o.push("".into());

        let tiles = treemap_tiles(closures, host);
        if !tiles.is_empty() {
            let file = format!("closures-{}.svg", sanitize(host));
            let caption = format!("{host} closure by package");
            out.write_auto(&src.join(&file), &chart::treemap(&caption, &tiles, style))?;
            o.push(format!("![{caption}](./{file})"));
            o.push("".into());
        }

        o.push("Largest single paths:".into());
        o.push("".into());
        o.push("| Package | Size |".into());
        o.push("|---|---|".into());
        for p in h.largest(TOP_PATHS) {
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
