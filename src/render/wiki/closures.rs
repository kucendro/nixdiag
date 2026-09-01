//! The Closures page: what each host's system actually weighs, and how much
//! of that weight the fleet shares.
//!
//! Opt-in and mode B only — nar sizes exist only for realised paths, so this
//! page is present exactly when `mkDocs { closures = …; }` supplied the data.

use super::super::chart::{self, Band, Row, Tile};
use super::super::d2::D2Style;
use super::super::out::{Out, MD_MARKER};
use crate::closures::{Closures, HostClosure};
use crate::facts::Facts;
use crate::util::{human_count, human_size, package_name, sanitize, store_name};
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

/// Biggest contributors listed per host. Enough to see what dominates without
/// turning the page into a store dump.
const TOP_PATHS: usize = 10;

/// Treemap tiles before the tail is folded into one. A real closure has a few
/// hundred packages, and past this many the rectangles are thinner than their
/// own labels.
const TREEMAP_TILES: usize = 24;

pub(super) fn page_closures(
    out: &mut Out,
    src: &Path,
    facts: &Facts,
    closures: &Closures,
    style: &D2Style,
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
        "NixOS hosts only — a darwin system cannot be built from Linux. A host \
         shown as — was not measured; a host serving these docs cannot measure \
         itself, since the docs would then depend on a system containing them."
            .into(),
        "".into(),
    ];
    // The chart is nixdiag's own SVG, not d2's, so it is written whatever
    // `--no-svg` says: that flag exists because d2 needs a binary on PATH and
    // its bytes move with its version, and neither is true here. Being an
    // `Auto` file, it is covered by the drift gate like the Markdown.
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

        // Single paths, where the treemap tiles are packages: the two differ
        // wherever one package ships several outputs.
        o.push("Largest single paths:".into());
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

/// One bar per host, stacked by how widely its paths are held.
///
/// An unmeasured host keeps its row and loses its bar, for the same reason it
/// keeps its `—` in the table: silently dropping it would make the picture
/// read as the whole fleet.
fn bar_rows(closures: &Closures, hosts: &[(&str, Option<&HostClosure>)]) -> Vec<Row> {
    // Measured against the same set `split` divides by, not against the rows:
    // with one measured host every path is trivially held by "all" of them, so
    // the split says nothing and the bar is drawn plain.
    let comparable = closures.hosts.len() > 1;
    hosts
        .iter()
        .map(|(host, closure)| match closure {
            Some(h) if comparable => {
                let s = closures.split(host);
                Row {
                    label: (*host).to_string(),
                    // Zero-valued bands draw nothing and claim no legend
                    // entry, so a fleet too small for one is handled here.
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
                note: "not measured".into(),
            },
        })
        .collect()
}

/// Treemap tiles for one host: package name and how widely it is held, summed
/// over every store path that folds to that name, with the long tail folded
/// into a single tile so it is visible without being drawn.
fn treemap_tiles(closures: &Closures, host: &str) -> Vec<Tile> {
    let n = closures.hosts.len();
    let band = |count: usize| match count {
        _ if n < 2 => Band::Solid,
        c if c >= n => Band::Shared,
        1 => Band::Unique,
        _ => Band::Partial,
    };

    // Keyed on the holder count as well as the name, so a tile never averages
    // two bands: one package's outputs are almost always held alike, and when
    // they are not, saying so is the honest answer.
    let mut groups: BTreeMap<(&str, usize), u64> = BTreeMap::new();
    for (path, size, count) in closures.path_shares(host) {
        *groups
            .entry((package_name(store_name(path)), count))
            .or_default() += size;
    }
    let mut v: Vec<((&str, usize), u64)> = groups.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let mut tiles: Vec<Tile> = v
        .iter()
        .take(TREEMAP_TILES)
        .map(|((name, count), size)| Tile {
            label: (*name).to_string(),
            value: *size,
            band: band(*count),
        })
        .collect();
    let rest: u64 = v.iter().skip(TREEMAP_TILES).map(|(_, s)| s).sum();
    if rest > 0 {
        tiles.push(Tile {
            label: format!("{} more", human_count(v.len() - TREEMAP_TILES)),
            value: rest,
            band: Band::Rest,
        });
    }
    tiles
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

    #[test]
    fn a_lone_measured_host_gets_one_plain_band() {
        let c = closures();
        let rows = bar_rows(&c, &[("nas", c.hosts.get("nas")), ("edge", None)]);
        assert_eq!(rows[0].bands, vec![(Band::Solid, 1024)]);
        assert_eq!(rows[0].note, "1.0 KiB");
        assert!(rows[1].bands.is_empty());
        assert_eq!(rows[1].note, "not measured");
    }

    #[test]
    fn treemap_tiles_fold_a_packages_outputs_together() {
        let mut hosts = IndexMap::new();
        let path = |n: &str, name: &str, size| ClosurePath {
            path: format!("/nix/store/0000000000000000000000000000000{n}-{name}"),
            nar_size: size,
        };
        hosts.insert(
            "nas".to_string(),
            HostClosure {
                paths: vec![
                    path("a", "glibc-2.42-67", 100),
                    path("b", "glibc-2.42-67-bin", 40),
                    path("c", "linux-6.12.9", 300),
                ],
            },
        );
        let c = Closures { schema: 1, hosts };
        let tiles = treemap_tiles(&c, "nas");
        let seen: Vec<(&str, u64)> = tiles.iter().map(|t| (t.label.as_str(), t.value)).collect();
        // Two outputs of glibc are one 140-byte tile, and it sorts under linux.
        assert_eq!(seen, vec![("linux", 300), ("glibc", 140)]);
        // One measured host, so nothing to compare and no legend to earn.
        assert!(tiles.iter().all(|t| t.band == Band::Solid), "{seen:?}");
    }

    #[test]
    fn the_treemap_tail_folds_into_one_counted_tile() {
        let mut hosts = IndexMap::new();
        // TREEMAP_TILES big ones plus three stragglers.
        let paths = (0..TREEMAP_TILES + 3)
            .map(|i| ClosurePath {
                path: format!("/nix/store/0000000000000000000000000000{i:04}-pkg{i:03}-1.0"),
                nar_size: if i < TREEMAP_TILES { 1000 } else { 7 },
            })
            .collect();
        hosts.insert("nas".to_string(), HostClosure { paths });
        let c = Closures { schema: 1, hosts };
        let tiles = treemap_tiles(&c, "nas");
        assert_eq!(tiles.len(), TREEMAP_TILES + 1);
        let last = tiles.last().unwrap();
        assert_eq!(last.label, "3 more");
        assert_eq!(last.value, 21);
        assert_eq!(last.band, Band::Rest);
    }

    /// Three hosts is the smallest fleet where a path can be held by some but
    /// not all, so it is the only shape that exercises every band.
    #[test]
    fn three_hosts_stack_all_three_bands() {
        let mut hosts = IndexMap::new();
        let path = |n: &str, size| ClosurePath {
            path: format!("/nix/store/0000000000000000000000000000000{n}-p"),
            nar_size: size,
        };
        hosts.insert(
            "a".to_string(),
            HostClosure {
                paths: vec![path("a", 100), path("b", 20), path("c", 3)],
            },
        );
        hosts.insert(
            "b".to_string(),
            HostClosure {
                paths: vec![path("a", 100), path("b", 20)],
            },
        );
        hosts.insert(
            "c".to_string(),
            HostClosure {
                paths: vec![path("a", 100)],
            },
        );
        let c = Closures { schema: 1, hosts };
        let rows = bar_rows(&c, &[("a", c.hosts.get("a"))]);
        assert_eq!(
            rows[0].bands,
            vec![(Band::Shared, 100), (Band::Partial, 20), (Band::Unique, 3)]
        );
    }
}
