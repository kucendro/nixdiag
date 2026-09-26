mod dups;
#[cfg(test)]
mod tests;

pub use dups::Dup;

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
    pub locked: Option<Locked>,
}

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

impl Lock {
    pub fn read(repo_root: &Path) -> Option<Lock> {
        let text = std::fs::read_to_string(repo_root.join("flake.lock")).ok()?;
        match serde_json::from_str::<Lock>(&text) {
            Ok(lock) => {
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

    fn resolve(&self, path: &[String]) -> Option<String> {
        let mut at = self.root.clone();
        for seg in path {
            let next = self.nodes.get(&at)?.inputs.get(seg)?;
            at = match next {
                InputRef::Node(n) => n.clone(),
                InputRef::Follows(p) if p != path => self.resolve(p)?,
                InputRef::Follows(_) => return None,
            };
        }
        Some(at)
    }

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

    pub fn date_span(&self) -> Option<(i64, i64)> {
        let dates: Vec<i64> = self
            .inputs()
            .into_iter()
            .filter_map(|(_, l)| l.last_modified)
            .collect();
        Some((*dates.iter().min()?, *dates.iter().max()?))
    }
}
