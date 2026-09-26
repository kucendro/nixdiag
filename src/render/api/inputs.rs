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
