mod charts;
#[cfg(test)]
mod tests;

use super::super::chart;
use super::{code, size_paths, table, Page, Wiki};
use crate::closures::{Closures, HostClosure};
use crate::conf::files::{chart as svg, page};
use crate::conf::limits::TOP_PATHS;
use crate::human::{Bytes, Count};
use crate::text::wiki::{closures as t, KV, NONE};
use crate::util::{sanitize, store_name};
use anyhow::Result;
use charts::{bar_rows, treemap_tiles};

pub(super) struct ClosuresPage;

impl Page for ClosuresPage {
    fn file(&self) -> &'static str {
        page::CLOSURES
    }

    fn title(&self) -> &'static str {
        t::TITLE
    }

    fn body(&self, w: &Wiki) -> Result<Option<Vec<String>>> {
        w.closures.map(|c| body(w, c)).transpose()
    }
}

fn body(w: &Wiki, closures: &Closures) -> Result<Vec<String>> {
    let hosts: Vec<(&str, Option<&HostClosure>)> = w
        .facts
        .hosts
        .iter()
        .filter(|(_, h)| h.as_nixos().is_some())
        .map(|(n, _)| (n.as_str(), closures.hosts.get(n.as_str())))
        .collect();

    let mut o = Vec::new();
    if !hosts.is_empty() {
        let bars = chart::bars(&bar_rows(closures, &hosts), w.style)?;
        w.src.write(svg::CLOSURES, &bars)?;
        o.push(t::image(t::CHART_CAPTION, svg::CLOSURES));
    }
    o.push(table(t::HEAD, summary_rows(closures, &hosts)));

    let measured = hosts.iter().filter(|(_, c)| c.is_some()).count();
    if measured > 1 {
        let (shared, deduped) = (closures.shared(), closures.deduped());
        let naive = closures.naive_sum();
        let saved = naive.saturating_sub(deduped.size);
        o.push(t::FLEET.into());
        o.push(table(
            KV,
            [
                [t::SHARED.into(), size_paths(&shared)],
                [t::DEDUPED.into(), size_paths(&deduped)],
                [t::SUM.into(), Bytes(naive).to_string()],
                [t::SAVED.into(), Bytes(saved).to_string()],
            ],
        ));
    }

    for (host, closure) in &hosts {
        let Some(h) = closure else { continue };
        o.push(t::host(host));
        if closures.served.iter().any(|s| s == *host) {
            o.push(t::SERVED.into());
        }

        let tiles = treemap_tiles(closures, host);
        if !tiles.is_empty() {
            let file = svg::host_closure(&sanitize(host));
            let caption = t::treemap_caption(host);
            w.src.write(&file, &chart::treemap(&tiles, w.style)?)?;
            o.push(t::image(&caption, &file));
        }

        let rows = h
            .largest(TOP_PATHS)
            .into_iter()
            .map(|p| [code(store_name(&p.path)), Bytes(p.nar_size).to_string()]);
        o.push(t::LARGEST.into());
        o.push(table(t::LARGEST_HEAD, rows));
    }
    Ok(o)
}

fn summary_rows(closures: &Closures, hosts: &[(&str, Option<&HostClosure>)]) -> Vec<[String; 4]> {
    let row = |host: &str, closure: Option<&HostClosure>| {
        let Some(h) = closure else {
            return [code(host), NONE.into(), NONE.into(), NONE.into()];
        };
        let (total, unique) = (h.total(), closures.unique(host).size);
        let (size, paths) = (Bytes(total.size), Count(total.paths));
        [
            code(host),
            size.to_string(),
            paths.to_string(),
            Bytes(unique).to_string(),
        ]
    };
    hosts.iter().map(|(host, c)| row(host, *c)).collect()
}
