use super::*;
use crate::conf::limits::TREEMAP_TILES;
use crate::render::chart::Band;

fn closures() -> Closures {
    Closures::of(vec![(
        "nas",
        vec![("/nix/store/0000000000000000000000000000000a-bash-5.2", 1024)],
    )])
}

#[test]
fn unselected_hosts_still_get_a_row() {
    let c = closures();
    let nas = c.hosts.get("nas");
    let rows = summary_rows(&c, &[("nas", nas), ("edge", None)]);
    assert_eq!(rows[0][..2], [code("nas"), "1.0 KiB".into()], "{rows:?}");
    assert_eq!(
        rows[1],
        [code("edge"), NONE.into(), NONE.into(), NONE.into()]
    );
}

#[test]
fn no_nixos_hosts_at_all_renders_a_placeholder() {
    let table = table(t::HEAD, summary_rows(&closures(), &[]));
    assert_eq!(table.lines().last().unwrap().matches(NONE).count(), 4);
}

#[test]
fn a_lone_measured_host_gets_one_plain_band() {
    let c = closures();
    let rows = bar_rows(&c, &[("nas", c.hosts.get("nas")), ("edge", None)]);
    assert_eq!(rows[0].bands, vec![(Band::Solid, 1024)]);
    assert_eq!(rows[0].note, "1.0 KiB");
    assert!(rows[1].bands.is_empty());
    assert_eq!(rows[1].note, crate::text::wiki::NOT_MEASURED);
}

#[test]
fn treemap_tiles_fold_a_packages_outputs_together() {
    let c = Closures::of(vec![(
        "nas",
        vec![
            ("/nix/store/a-glibc-2.42-67", 100),
            ("/nix/store/b-glibc-2.42-67-bin", 40),
            ("/nix/store/c-linux-6.12.9", 300),
        ],
    )]);
    let tiles = treemap_tiles(&c, "nas");
    let seen: Vec<(&str, u64)> = tiles.iter().map(|t| (t.label.as_str(), t.value)).collect();
    assert_eq!(seen, vec![("linux", 300), ("glibc", 140)]);
    assert!(tiles.iter().all(|t| t.band == Band::Solid), "{seen:?}");
}

#[test]
fn the_treemap_tail_folds_into_one_counted_tile() {
    let names: Vec<String> = (0..TREEMAP_TILES + 3)
        .map(|i| format!("/nix/store/0000000000000000000000000000{i:04}-pkg{i:03}-1.0"))
        .collect();
    let paths = names.iter().enumerate().map(|(i, p)| {
        let size = if i < TREEMAP_TILES { 1000 } else { 7 };
        (p.as_str(), size)
    });
    let c = Closures::of(vec![("nas", paths.collect())]);
    let tiles = treemap_tiles(&c, "nas");
    assert_eq!(tiles.len(), TREEMAP_TILES + 1);
    let last = tiles.last().unwrap();
    assert_eq!(last.label, fill(t::MORE, &[("count", "3")]));
    assert_eq!(last.value, 21);
    assert_eq!(last.band, Band::Rest);
}

#[test]
fn three_hosts_stack_all_three_bands() {
    let c = Closures::of(vec![
        ("a", vec![("a-p", 100), ("b-p", 20), ("c-p", 3)]),
        ("b", vec![("a-p", 100), ("b-p", 20)]),
        ("c", vec![("a-p", 100)]),
    ]);
    let rows = bar_rows(&c, &[("a", c.hosts.get("a"))]);
    assert_eq!(
        rows[0].bands,
        vec![(Band::Shared, 100), (Band::Partial, 20), (Band::Unique, 3)]
    );
}
