use super::*;

const LOCK: &str = r#"{
  "nodes": {
    "root":    { "inputs": { "nixpkgs": "nixpkgs", "stylix": "stylix", "utils": "utils" } },
    "nixpkgs": { "locked": { "type": "github", "owner": "nixos", "repo": "nixpkgs",
                             "rev": "56c02bc00adcf003215cc4bd996d6efaf4cff188",
                             "lastModified": 1787498568 } },
    "stylix":  { "inputs": { "nixpkgs": "nixpkgs_2", "systems": ["utils"] },
                 "locked": { "type": "github", "owner": "danth", "repo": "stylix",
                             "rev": "aaaaaaaaaaaaaaaa" } },
    "nixpkgs_2": { "locked": { "type": "github", "owner": "NixOS", "repo": "nixpkgs",
                               "rev": "89570f24b6c1a91a5b0b3a1a3a4a4a4a4a4a4a4a" } },
    "utils":   { "locked": { "type": "github", "owner": "numtide", "repo": "flake-utils",
                             "rev": "11707dc2f618dd54ca8739b309ec4fc024de578b" } },
    "utils_2": { "locked": { "type": "github", "owner": "numtide", "repo": "flake-utils",
                             "rev": "11707dc2f618dd54ca8739b309ec4fc024de578b" } }
  },
  "root": "root",
  "version": 7
}"#;

fn lock() -> Lock {
    serde_json::from_str(LOCK).unwrap()
}

#[test]
fn root_has_no_locked() {
    let l = lock();
    assert!(l.nodes["root"].locked.is_none());
    assert!(l.nodes["nixpkgs"].locked.is_some());
}

#[test]
fn input_ref_parses_both_shapes() {
    let l = lock();
    let stylix = &l.nodes["stylix"];
    assert!(matches!(stylix.inputs["nixpkgs"], InputRef::Node(_)));
    assert!(matches!(stylix.inputs["systems"], InputRef::Follows(_)));
}

#[test]
fn follows_resolves_through_the_root() {
    let l = lock();
    assert_eq!(l.resolve(&["utils".to_string()]), Some("utils".to_string()));
}

#[test]
fn follows_edges_are_marked() {
    let l = lock();
    let e = l.edges();
    assert!(e.contains(&("stylix".into(), "systems".into(), "utils".into(), true)));
    assert!(e.contains(&("stylix".into(), "nixpkgs".into(), "nixpkgs_2".into(), false)));
}

#[test]
fn identity_folds_forge_case() {
    let l = lock();
    let a = l.nodes["nixpkgs"].locked.as_ref().unwrap();
    let b = l.nodes["nixpkgs_2"].locked.as_ref().unwrap();
    assert_eq!(a.identity(), b.identity());
}

#[test]
fn diamond_beats_redundancy_in_the_ordering() {
    let dups = lock().duplicates();
    assert_eq!(dups.len(), 2);
    assert!(dups[0].is_diamond());
    assert_eq!(dups[0].source, "github:nixos/nixpkgs");
    assert_eq!(dups[0].nodes(), vec!["nixpkgs", "nixpkgs_2"]);
    assert!(!dups[1].is_diamond());
    assert_eq!(dups[1].nodes(), vec!["utils", "utils_2"]);
}

#[test]
fn dedup_target_is_the_roots_own_input_name() {
    let l = lock();
    let id = l.nodes["nixpkgs_2"].locked.as_ref().unwrap().identity();
    assert_eq!(l.root_input_for(&id), Some("nixpkgs".to_string()));
    assert_eq!(
        l.parents_of("nixpkgs_2"),
        vec![("stylix".into(), "nixpkgs".into())]
    );
}

#[test]
fn root_inputs_are_the_ones_this_flake_declares() {
    let l = lock();
    let roots = l.root_inputs();
    assert!(roots.contains("nixpkgs") && roots.contains("stylix") && roots.contains("utils"));
    assert!(!roots.contains("nixpkgs_2"), "{roots:?}");
}

#[test]
fn missing_lock_is_not_an_error() {
    assert!(Lock::read(Path::new("/nonexistent-nixdiag-test")).is_none());
}
