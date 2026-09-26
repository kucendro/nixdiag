use super::{InputRef, Lock};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Dup {
    pub identity: String,
    pub source: String,
    pub revs: Vec<(String, Vec<String>)>,
}

impl Dup {
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
