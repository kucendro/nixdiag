use crate::closures::{Closures, HostClosure};
use crate::render::chart::{Band, Row, Tile};
use crate::text::fill;
use crate::text::wiki::closures as t;
use crate::util::{human_count, human_size};

pub(super) const TREEMAP_TILES: usize = 24;

pub(super) fn bar_rows(closures: &Closures, hosts: &[(&str, Option<&HostClosure>)]) -> Vec<Row> {
    let comparable = closures.hosts.len() > 1;
    hosts
        .iter()
        .map(|(host, closure)| match closure {
            Some(h) if comparable => {
                let s = closures.split(host);
                Row {
                    label: (*host).to_string(),
                    bands: vec![
                        (Band::Shared, s.shared),
                        (Band::Partial, s.partial),
                        (Band::Unique, s.unique),
                    ],
                    note: human_size(h.total()),
                }
            }
            Some(h) => Row {
                label: (*host).to_string(),
                bands: vec![(Band::Solid, h.total())],
                note: human_size(h.total()),
            },
            None => Row {
                label: (*host).to_string(),
                bands: Vec::new(),
                note: t::NOT_MEASURED.into(),
            },
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
        .map(|(name, size, count)| Tile {
            label: name.clone(),
            value: *size,
            band: band(*count),
        })
        .collect();
    let rest: u64 = v.iter().skip(TREEMAP_TILES).map(|(_, s, _)| s).sum();
    if rest > 0 {
        tiles.push(Tile {
            label: fill(t::MORE, &[("count", &human_count(v.len() - TREEMAP_TILES))]),
            value: rest,
            band: Band::Rest,
        });
    }
    tiles
}
