use crate::closures::{Closures, HostClosure};
use crate::conf::limits::TREEMAP_TILES;
use crate::render::chart::{Band, Row, Tile};
use crate::text::fill;
use crate::text::wiki::closures as t;
use crate::util::{human_count, human_size};

pub(super) fn bar_rows(closures: &Closures, hosts: &[(&str, Option<&HostClosure>)]) -> Vec<Row> {
    let comparable = closures.hosts.len() > 1;
    hosts
        .iter()
        .map(|(host, closure)| {
            let label = (*host).to_string();
            let Some(h) = closure else {
                let note = t::NOT_MEASURED.into();
                return Row {
                    label,
                    bands: Vec::new(),
                    note,
                };
            };
            let size = h.total().size;
            let bands = if comparable {
                let s = closures.split(host);
                vec![
                    (Band::Shared, s.shared),
                    (Band::Partial, s.partial),
                    (Band::Unique, s.unique),
                ]
            } else {
                vec![(Band::Solid, size)]
            };
            Row {
                label,
                bands,
                note: human_size(size),
            }
        })
        .collect()
}

pub(super) fn treemap_tiles(closures: &Closures, host: &str) -> Vec<Tile> {
    let n = closures.hosts.len();
    let band = |count: usize| match count {
        _ if n < 2 => Band::Solid,
        c if c >= n => Band::Shared,
        1 => Band::Unique,
        _ => Band::Partial,
    };

    let v = closures.package_shares(host);

    let mut tiles: Vec<Tile> = v
        .iter()
        .take(TREEMAP_TILES)
        .map(|s| Tile {
            label: s.name.clone(),
            value: s.size,
            band: band(s.holders),
        })
        .collect();
    let rest: u64 = v.iter().skip(TREEMAP_TILES).map(|s| s.size).sum();
    if rest > 0 {
        tiles.push(Tile {
            label: fill(t::MORE, &[("count", &human_count(v.len() - TREEMAP_TILES))]),
            value: rest,
            band: Band::Rest,
        });
    }
    tiles
}
