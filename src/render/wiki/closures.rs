mod charts;
#[cfg(test)]
mod tests;

use super::super::chart;
use super::{table, Wiki};
use crate::closures::{Closures, HostClosure};
use crate::conf::files::{chart as svg, page};
use crate::conf::limits::TOP_PATHS;
use crate::human::{Bytes, Count};
use crate::text::fill;
use crate::text::wiki::closures as t;
use crate::util::{sanitize, store_name};
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
    o.push(table(&t::HEAD, &summary_rows(closures, &hosts)));

    let measured = hosts.iter().filter(|(_, c)| c.is_some()).count();
    if measured > 1 {
        let (shared, deduped) = (closures.shared(), closures.deduped());
        let naive = closures.naive_sum();
        o.push(fill(
            t::FLEET,
            &[
                ("shared", &Bytes(shared.size).to_string()),
                ("shared_paths", &Count(shared.paths).to_string()),
                ("deduped", &Bytes(deduped.size).to_string()),
                ("deduped_paths", &Count(deduped.paths).to_string()),
                ("sum", &Bytes(naive).to_string()),
                (
                    "saved",
                    &Bytes(naive.saturating_sub(deduped.size)).to_string(),
                ),
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

        let rows: Vec<String> = h
            .largest(TOP_PATHS)
            .iter()
            .map(|p| {
                fill(
                    t::LARGEST_ROW,
                    &[
                        ("package", store_name(&p.path)),
                        ("size", &Bytes(p.nar_size).to_string()),
                    ],
                )
            })
            .collect();
        o.push(t::LARGEST.into());
        o.push(table(&t::LARGEST_HEAD, &rows));
    }

    w.page(page::CLOSURES, &o)
}

fn summary_rows(closures: &Closures, hosts: &[(&str, Option<&HostClosure>)]) -> Vec<String> {
    hosts
        .iter()
        .map(|(host, closure)| match closure {
            Some(h) => {
                let total = h.total();
                fill(
                    t::ROW,
                    &[
                        ("host", host),
                        ("closure", &Bytes(total.size).to_string()),
                        ("paths", &Count(total.paths).to_string()),
                        ("unique", &Bytes(closures.unique(host).size).to_string()),
                    ],
                )
            }
            None => fill(t::ROW_UNMEASURED, &[("host", host)]),
        })
        .collect()
}
