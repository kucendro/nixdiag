//! The Inputs page: the flake's supply chain, plus the two duplicate signals.
//!
//! A repo pulled in twice is reported as one of two different things, because
//! they need different reactions: several *revisions* of one repo is a
//! correctness risk, while one revision under several node names is only a
//! redundant fetch.

use super::super::out::{Out, MD_MARKER};
use crate::source::flakelock::{Dup, Lock};
use crate::util::human_date;
use anyhow::Result;
use std::path::Path;

/// "this flake" for the root, otherwise the node and the name it uses.
fn pulled_in_by(lock: &Lock, node: &str) -> String {
    let parents = lock.parents_of(node);
    if parents.is_empty() {
        return "—".into();
    }
    parents
        .iter()
        .map(|(parent, input)| {
            if *parent == lock.root {
                "this flake".to_string()
            } else if parent == input {
                format!("`{parent}`")
            } else {
                format!("`{parent}` (as `{input}`)")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn diamond(o: &mut Vec<String>, lock: &Lock, d: &Dup) {
    o.push(format!(
        "`{}` is locked at **{} revisions**, so every copy is fetched and \
         evaluated separately:",
        d.source,
        d.revs.len()
    ));
    o.push("".into());
    o.push("| Rev | Node | Pulled in by |".into());
    o.push("|---|---|---|".into());
    for (rev, nodes) in &d.revs {
        let short: String = rev.chars().take(7).collect();
        for n in nodes {
            o.push(format!("| `{short}` | `{n}` | {} |", pulled_in_by(lock, n)));
        }
    }
    o.push("".into());

    // Suggest pointing the extra copies at whatever the root already pulls.
    let Some(target) = lock.root_input_for(&d.identity) else {
        return;
    };
    let mut fixes: Vec<String> = Vec::new();
    for n in d.nodes() {
        if n == target {
            continue;
        }
        for (parent, input) in lock.parents_of(n) {
            if parent == lock.root {
                continue;
            }
            fixes.push(format!(
                "inputs.{parent}.inputs.{input}.follows = \"{target}\";"
            ));
        }
    }
    if fixes.is_empty() {
        return;
    }
    fixes.sort();
    fixes.dedup();
    o.push(format!("Point the extra copies at `{target}`:"));
    o.push("".into());
    o.push("```nix".into());
    o.extend(fixes);
    o.push("```".into());
    o.push("".into());
}

pub(super) fn page_inputs(out: &mut Out, src: &Path, lock: &Lock) -> Result<()> {
    let from = out.root.join("inputs.svg");
    if from.exists() {
        let rel = src.join("inputs.svg");
        std::fs::create_dir_all(out.root.join(src))?;
        std::fs::copy(&from, out.root.join(&rel))?;
        out.record_svg(&rel);
    }

    let mut o: Vec<String> = vec![
        MD_MARKER.into(),
        "".into(),
        "# Inputs".into(),
        "".into(),
        "What this flake depends on, read from `flake.lock`. Dashed edges are \
         `follows`, which is what *removes* a duplicate rather than adding one."
            .into(),
        "".into(),
        // Emitted whether or not the SVG was rendered: `nixdiag check` runs
        // with --no-svg, and the Markdown must not differ between the two.
        "![Input graph](./inputs.svg)".into(),
        "".into(),
        "| Input | Source | Rev | Locked |".into(),
        "|---|---|---|---|".into(),
    ];
    for (name, locked) in lock.inputs() {
        let date = locked
            .last_modified
            .map(human_date)
            .unwrap_or_else(|| "—".into());
        o.push(format!(
            "| `{name}` | `{}` | `{}` | {date} |",
            locked.source(),
            locked.short_rev()
        ));
    }
    if lock.inputs().is_empty() {
        o.push("| — | — | — | — |".into());
    }
    o.push("".into());

    let dups = lock.duplicates();
    let (diamonds, redundant): (Vec<&Dup>, Vec<&Dup>) = dups.iter().partition(|d| d.is_diamond());

    if !diamonds.is_empty() {
        o.push("## Duplicate inputs".into());
        o.push("".into());
        for d in diamonds {
            diamond(&mut o, lock, d);
        }
    }

    if !redundant.is_empty() {
        o.push("## Redundant inputs".into());
        o.push("".into());
        o.push(
            "Locked at a single revision, but reached under more than one node \
             name. Harmless, though a `follows` would drop the extra fetch."
                .into(),
        );
        o.push("".into());
        for d in redundant {
            let nodes = d
                .nodes()
                .iter()
                .map(|n| format!("`{n}`"))
                .collect::<Vec<_>>()
                .join(", ");
            o.push(format!("- `{}` — {nodes}", d.source));
        }
        o.push("".into());
    }

    out.write_auto(&src.join("inputs.md"), &o.join("\n"))
}
