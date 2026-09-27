mod charts;
#[cfg(test)]
mod tests;

use super::super::chart;
use super::Wiki;
use crate::closures::{Closures, HostClosure};
use crate::conf::files::{chart as svg, page};
use crate::conf::limits::TOP_PATHS;
use crate::text::fill;
use crate::text::wiki::closures as t;
use crate::util::{human_count, human_size, sanitize, store_name};
use anyhow::Result;
use charts::{bar_rows, treemap_tiles};

pub(super) fn page_closures(w: &Wiki, closures: &Closures) -> Result<()> {
    let hosts: Vec<(&str, Option<&HostClosure>)> = w
        .facts
        .hosts
        .iter()
        .filter(|(_, h)| h.as_nixos().is_some())
        .map(|(n, _)| (n.as_str(), closures.hosts.get(n.as_str())))
        .collect();

    let mut o = vec![t::TITLE.to_string()];
    if !hosts.is_empty() {
        let bars = chart::bars(t::CHART_CAPTION, &bar_rows(closures, &hosts), w.style);
        w.src.write(svg::CLOSURES, &bars)?;
        o.push(fill(t::CHART, &[("caption", t::CHART_CAPTION)]));
    }
    o.push(fill(
        t::TABLE,
        &[("rows", &summary_rows(closures, &hosts).join("\n"))],
    ));

    let measured = hosts.iter().filter(|(_, c)| c.is_some()).count();
    if measured > 1 {
        let shared = closures.shared();
        let shared_size: u64 = shared.iter().map(|(_, s)| s).sum();
        let (dedup_n, dedup_size) = closures.deduped();
        let naive = closures.naive_sum();
        o.push(fill(
            t::FLEET,
            &[
                ("shared", &human_size(shared_size)),
                ("shared_paths", &human_count(shared.len())),
                ("deduped", &human_size(dedup_size)),
                ("deduped_paths", &human_count(dedup_n)),
                ("sum", &human_size(naive)),
                ("saved", &human_size(naive.saturating_sub(dedup_size))),
            ],
        ));
    }

    for (host, closure) in &hosts {
        let Some(h) = closure else { continue };
        o.push(fill(t::HOST, &[("host", host)]));
        if closures.served.iter().any(|s| s == *host) {
            o.push(t::SERVED.into());
        }

        let tiles = treemap_tiles(closures, host);
        if !tiles.is_empty() {
            let file = fill(svg::HOST_CLOSURE, &[("host", &sanitize(host))]);
            let caption = fill(t::TREEMAP_CAPTION, &[("host", host)]);
            w.src
                .write(&file, &chart::treemap(&caption, &tiles, w.style))?;
            o.push(fill(t::TREEMAP, &[("caption", &caption), ("file", &file)]));
        }

        let mut rows: Vec<String> = h
            .largest(TOP_PATHS)
            .iter()
            .map(|p| {
                fill(
                    t::LARGEST_ROW,
                    &[
                        ("package", store_name(&p.path)),
                        ("size", &human_size(p.nar_size)),
                    ],
                )
            })
            .collect();
        if rows.is_empty() {
            rows.push(t::LARGEST_EMPTY.into());
        }
        o.push(fill(t::LARGEST, &[("rows", &rows.join("\n"))]));
    }

    w.page(page::CLOSURES, &o)
}

fn summary_rows(closures: &Closures, hosts: &[(&str, Option<&HostClosure>)]) -> Vec<String> {
    if hosts.is_empty() {
        return vec![t::EMPTY.into()];
    }
    hosts
        .iter()
        .map(|(host, closure)| match closure {
            Some(h) => {
                let unique: u64 = closures.unique(host).iter().map(|(_, s)| s).sum();
                fill(
                    t::ROW,
                    &[
                        ("host", host),
                        ("closure", &human_size(h.total())),
                        ("paths", &human_count(h.len())),
                        ("unique", &human_size(unique)),
                    ],
                )
            }
            None => fill(t::ROW_UNMEASURED, &[("host", host)]),
        })
        .collect()
}
