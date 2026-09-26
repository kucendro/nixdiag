use super::super::chart::{self, Mark};
use super::super::d2::D2Style;
use super::super::out::{Out, MD_MARKER};
use crate::source::flakelock::{Dup, Lock};
use crate::util::human_date;
use anyhow::Result;
use std::path::Path;

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

fn lock_dates(
    o: &mut Vec<String>,
    out: &mut Out,
    src: &Path,
    lock: &Lock,
    style: &D2Style,
) -> Result<()> {
    let roots = lock.root_inputs();
    let marks: Vec<Mark> = lock
        .inputs()
        .into_iter()
        .map(|(name, locked)| Mark {
            label: name.clone(),
            at: locked.last_modified,
            direct: roots.contains(name.as_str()),
            note: locked
                .last_modified
                .map(human_date)
                .unwrap_or_else(|| "—".into()),
        })
        .collect();
    let Some((lo, hi)) = lock.date_span() else {
        return Ok(());
    };

    let svg = chart::timeline("Locked inputs by date, oldest first", &marks, style);
    out.write_auto(&src.join("inputs-timeline.svg"), &svg)?;

    o.push("## Lock dates".into());
    o.push("".into());
    o.push("![Input dates](./inputs-timeline.svg)".into());
    o.push("".into());
    o.push(
        "`lastModified` is a fixed integer in the lock, not a clock read: this \
         is the *spread*, not a claim about today."
            .into(),
    );
    o.push("".into());
    let days = (hi - lo) / 86_400;
    if days > 0 {
        o.push(format!(
            "**{days} days** separate the oldest input from the newest."
        ));
        o.push("".into());
    }
    Ok(())
}

pub(super) fn page_inputs(out: &mut Out, src: &Path, lock: &Lock, style: &D2Style) -> Result<()> {
    let from = out.root.join("inputs.svg");
    if from.exists() {
        let rel = src.join("inputs.svg");
        std::fs::create_dir_all(out.root.join(src))?;
        std::fs::copy(&from, out.root.join(&rel))?;
    }

    let mut o: Vec<String> = vec![
        MD_MARKER.into(),
        "".into(),
        "# Inputs".into(),
        "".into(),
        "Dashed edges are `follows`, which *removes* a duplicate.".into(),
        "".into(),
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

    lock_dates(&mut o, out, src, lock, style)?;

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
            "One revision under several node names. Harmless; a `follows` drops \
             the extra fetch."
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
