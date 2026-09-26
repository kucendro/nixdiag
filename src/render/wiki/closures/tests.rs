use super::charts::TREEMAP_TILES;
use super::*;
use crate::closures::ClosurePath;
use crate::render::chart::Band;
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
        rows.contains(&fill(t::ROW_UNMEASURED, &[("host", "edge")])),
        "{rows:?}"
    );
}

#[test]
fn no_nixos_hosts_at_all_renders_a_placeholder() {
    let rows = summary_rows(&closures(), &[]);
    assert_eq!(rows.last().unwrap(), t::EMPTY);
}

#[test]
fn a_lone_measured_host_gets_one_plain_band() {
    let c = closures();
    let rows = bar_rows(&c, &[("nas", c.hosts.get("nas")), ("edge", None)]);
    assert_eq!(rows[0].bands, vec![(Band::Solid, 1024)]);
    assert_eq!(rows[0].note, "1.0 KiB");
    assert!(rows[1].bands.is_empty());
    assert_eq!(rows[1].note, t::NOT_MEASURED);
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
    assert_eq!(seen, vec![("linux", 300), ("glibc", 140)]);
    assert!(tiles.iter().all(|t| t.band == Band::Solid), "{seen:?}");
}

#[test]
fn the_treemap_tail_folds_into_one_counted_tile() {
    let mut hosts = IndexMap::new();
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
    assert_eq!(last.label, fill(t::MORE, &[("count", "3")]));
    assert_eq!(last.value, 21);
    assert_eq!(last.band, Band::Rest);
}

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
