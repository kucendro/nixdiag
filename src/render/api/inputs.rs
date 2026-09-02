//! `inputs.json` — the flake.lock graph.
//!
//! A plain file read: no eval, no realisation, no clock. `lastModified` is a
//! fixed integer in the lock, which is what keeps this document identical
//! between two builds of the same revision.

use crate::api::{self, Meta};
use crate::source::flakelock::Lock;

pub(super) fn build(meta: Meta, lock: &Lock) -> api::Inputs {
    let roots = lock.root_inputs();
    let nodes = lock
        .inputs()
        .into_iter()
        .map(|(name, locked)| api::InputNode {
            name: name.clone(),
            source: locked.source(),
            rev: locked.rev.clone(),
            last_modified: locked.last_modified,
            // Declared by the root flake, so `nix flake update` moves it.
            direct: roots.contains(name.as_str()),
        })
        .collect();

    let edges = lock
        .edges()
        .into_iter()
        .map(|(parent, input, child, follows)| api::InputEdge {
            from: parent,
            to: child,
            input,
            follows,
        })
        .collect();

    let duplicates = lock
        .duplicates()
        .into_iter()
        .map(|d| api::Duplicate {
            diamond: d.is_diamond(),
            // Only suggested when the root has an input to point at, so no
            // advice is invented for a repo the root never pulls.
            follows_target: lock.root_input_for(&d.identity),
            revisions: d
                .revs
                .into_iter()
                .map(|(rev, nodes)| api::RevGroup { rev, nodes })
                .collect(),
            source: d.source,
            identity: d.identity,
        })
        .collect();

    api::Inputs {
        meta,
        root: lock.root.clone(),
        nodes,
        edges,
        duplicates,
    }
}
