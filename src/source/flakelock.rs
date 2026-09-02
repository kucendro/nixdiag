//! The flake's own supply chain: `flake.lock` parsed into an input graph.
//!
//! A plain file read — no eval, no realisation, no clock, so this works
//! identically in both modes. `lastModified` is a fixed integer stored in the
//! lock, which is what keeps every rendered date deterministic.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Lock {
    pub root: String,
    pub nodes: BTreeMap<String, Node>,
    #[serde(default)]
    pub version: u32,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Node {
    pub inputs: BTreeMap<String, InputRef>,
    /// Absent on the root node, which is the flake itself.
    pub locked: Option<Locked>,
}

/// An input value is either a node name, or a `follows` path relative to the
/// root flake (`[ "stylix" "nixpkgs" ]`).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum InputRef {
    Node(String),
    Follows(Vec<String>),
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Locked {
    #[serde(rename = "type")]
    pub kind: String,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub rev: Option<String>,
    pub url: Option<String>,
    pub path: Option<String>,
    pub last_modified: Option<i64>,
    pub nar_hash: Option<String>,
}

impl Locked {
    /// Grouping key. Forge owner/repo are case-insensitive, and real locks do
    /// carry the same repo under different casing (`nixos/nixpkgs` next to
    /// `NixOS/nixpkgs`) — without folding case the most important duplicate a
    /// lock can hold is missed.
    pub fn identity(&self) -> String {
        match (&self.owner, &self.repo) {
            (Some(o), Some(r)) => {
                format!("{}:{}/{}", self.kind, o.to_lowercase(), r.to_lowercase())
            }
            _ => {
                let loc = self.url.as_deref().or(self.path.as_deref()).unwrap_or("?");
                format!("{}:{loc}", self.kind)
            }
        }
    }

    /// How the input is written in a flake, for display.
    pub fn source(&self) -> String {
        match (&self.owner, &self.repo) {
            (Some(o), Some(r)) => format!("{}:{o}/{r}", self.kind),
            _ => self
                .url
                .clone()
                .or_else(|| self.path.clone())
                .unwrap_or_else(|| self.kind.clone()),
        }
    }

    /// Revision, or the nar hash for sources that have no rev.
    pub fn version_id(&self) -> String {
        self.rev
            .clone()
            .or_else(|| self.nar_hash.clone())
            .unwrap_or_else(|| "—".into())
    }

    pub fn short_rev(&self) -> String {
        let v = self.version_id();
        v.chars().take(7).collect()
    }
}

/// One repo pulled in under more than one lock node.
#[derive(Debug)]
pub struct Dup {
    /// Case-folded grouping key, for looking the repo up again.
    pub identity: String,
    pub source: String,
    /// version id -> the nodes locked at it, both sorted
    pub revs: Vec<(String, Vec<String>)>,
}

impl Dup {
    /// More than one revision of the same repo: a correctness risk. A single
    /// revision under several node names is only redundancy.
    pub fn is_diamond(&self) -> bool {
        self.revs.len() > 1
    }

    pub fn nodes(&self) -> Vec<&str> {
        self.revs
            .iter()
            .flat_map(|(_, ns)| ns.iter().map(String::as_str))
            .collect()
    }
}

impl Lock {
    /// Parse `<repo>/flake.lock`. A flake without a lock is legitimate, so a
    /// missing file is `None` rather than an error.
    pub fn read(repo_root: &Path) -> Option<Lock> {
        let text = std::fs::read_to_string(repo_root.join("flake.lock")).ok()?;
        match serde_json::from_str::<Lock>(&text) {
            Ok(lock) => {
                // The nodes/inputs/locked shape has been stable across lock
                // versions 5-7, so an unfamiliar version is worth a word but
                // not a failure.
                if lock.version != 0 && lock.version != 7 {
                    eprintln!(
                        "note: flake.lock is version {}, expected 7 — reading it anyway",
                        lock.version
                    );
                }
                Some(lock)
            }
            Err(e) => {
                eprintln!("  ! flake.lock is not readable as a lock file, skipping: {e}");
                None
            }
        }
    }

    /// Follow a `follows` path from the root flake to the node it names.
    fn resolve(&self, path: &[String]) -> Option<String> {
        let mut at = self.root.clone();
        for seg in path {
            let next = self.nodes.get(&at)?.inputs.get(seg)?;
            at = match next {
                InputRef::Node(n) => n.clone(),
                // A follows pointing at a follows: resolve from the root
                // again, bounded by the path length so a malformed lock
                // cannot spin.
                InputRef::Follows(p) if p != path => self.resolve(p)?,
                InputRef::Follows(_) => return None,
            };
        }
        Some(at)
    }

    /// Every input edge as (parent node, input name, child node, is_follows).
    /// A `follows` is drawn distinctly because it is what *removes* a
    /// duplicate rather than adding one.
    pub fn edges(&self) -> Vec<(String, String, String, bool)> {
        let mut out = Vec::new();
        for (parent, node) in &self.nodes {
            for (name, r) in &node.inputs {
                let (child, follows) = match r {
                    InputRef::Node(n) => (Some(n.clone()), false),
                    InputRef::Follows(p) => (self.resolve(p), true),
                };
                if let Some(child) = child {
                    out.push((parent.clone(), name.clone(), child, follows));
                }
            }
        }
        out.sort();
        out
    }

    /// Nodes that pull `child` in directly, as (parent, input name). Follows
    /// edges are excluded: they express deduplication, not a second copy.
    pub fn parents_of(&self, child: &str) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = self
            .edges()
            .into_iter()
            .filter(|(_, _, c, follows)| c == child && !follows)
            .map(|(p, name, _, _)| (p, name))
            .collect();
        out.sort();
        out
    }

    /// Every input node except the root, sorted by name.
    pub fn inputs(&self) -> Vec<(&String, &Locked)> {
        let mut out: Vec<(&String, &Locked)> = self
            .nodes
            .iter()
            .filter(|(name, _)| *name != &self.root)
            .filter_map(|(name, n)| n.locked.as_ref().map(|l| (name, l)))
            .collect();
        out.sort_by(|a, b| a.0.cmp(b.0));
        out
    }

    /// Repos appearing under more than one node, worst first: real diamonds
    /// (several revisions) before mere redundancy.
    pub fn duplicates(&self) -> Vec<Dup> {
        let mut by_identity: BTreeMap<String, (String, BTreeMap<String, Vec<String>>)> =
            BTreeMap::new();
        for (name, locked) in self.inputs() {
            let e = by_identity
                .entry(locked.identity())
                .or_insert_with(|| (locked.source(), BTreeMap::new()));
            e.1.entry(locked.version_id())
                .or_default()
                .push(name.clone());
        }
        let mut dups: Vec<Dup> = by_identity
            .into_iter()
            .filter(|(_, (_, revs))| revs.values().map(Vec::len).sum::<usize>() > 1)
            .map(|(identity, (source, revs))| Dup {
                identity,
                source,
                revs: revs.into_iter().collect(),
            })
            .collect();
        dups.sort_by(|a, b| {
            b.is_diamond()
                .cmp(&a.is_diamond())
                .then(a.source.cmp(&b.source))
        });
        dups
    }

    /// The nodes the root flake declares itself.
    ///
    /// These are the inputs `nix flake update` moves; every other node is
    /// locked by whichever input pulled it in, so its date is that input's to
    /// move, not this flake's.
    pub fn root_inputs(&self) -> BTreeSet<String> {
        let Some(root) = self.nodes.get(&self.root) else {
            return BTreeSet::new();
        };
        root.inputs
            .values()
            .filter_map(|r| match r {
                InputRef::Node(n) => Some(n.clone()),
                InputRef::Follows(p) => self.resolve(p),
            })
            .collect()
    }

    /// Oldest and newest `lastModified` across the dated inputs.
    ///
    /// Lock arithmetic, never a clock read — which is the whole reason the
    /// timeline chart is deterministic and "overdue" is not rendered at all.
    /// `None` when nothing carries a date (every input a `path:`, or no
    /// inputs), because a span needs two ends.
    pub fn date_span(&self) -> Option<(i64, i64)> {
        let dates: Vec<i64> = self
            .inputs()
            .into_iter()
            .filter_map(|(_, l)| l.last_modified)
            .collect();
        Some((*dates.iter().min()?, *dates.iter().max()?))
    }

    /// The root's own input name for this repo, if it has one — the target a
    /// `follows` should point at.
    pub fn root_input_for(&self, identity: &str) -> Option<String> {
        let root = self.nodes.get(&self.root)?;
        for (name, r) in &root.inputs {
            let node = match r {
                InputRef::Node(n) => n.clone(),
                InputRef::Follows(p) => self.resolve(p)?,
            };
            let locked = self.nodes.get(&node).and_then(|n| n.locked.as_ref());
            if locked.map(|l| l.identity()) == Some(identity.to_string()) {
                return Some(name.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
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
        // The real-world case: nixos/nixpkgs and NixOS/nixpkgs are one repo.
        let l = lock();
        let a = l.nodes["nixpkgs"].locked.as_ref().unwrap();
        let b = l.nodes["nixpkgs_2"].locked.as_ref().unwrap();
        assert_eq!(a.identity(), b.identity());
    }

    #[test]
    fn diamond_beats_redundancy_in_the_ordering() {
        let dups = lock().duplicates();
        assert_eq!(dups.len(), 2);
        // nixpkgs: two revisions -> a real diamond, reported first.
        assert!(dups[0].is_diamond());
        assert_eq!(dups[0].source, "github:nixos/nixpkgs");
        assert_eq!(dups[0].nodes(), vec!["nixpkgs", "nixpkgs_2"]);
        // flake-utils: one revision under two names -> redundancy only.
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
        // Reached only through stylix, so a flake update here moves stylix,
        // not this node.
        assert!(!roots.contains("nixpkgs_2"), "{roots:?}");
    }

    #[test]
    fn missing_lock_is_not_an_error() {
        assert!(Lock::read(Path::new("/nonexistent-nixdiag-test")).is_none());
    }
}
