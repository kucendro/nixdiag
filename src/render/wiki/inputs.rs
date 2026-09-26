use super::super::chart::{self, Mark};
use super::super::d2::D2Style;
use super::super::out::Out;
use super::page;
use crate::source::flakelock::{Dup, Lock};
use crate::text::fill;
use crate::text::wiki::{inputs as t, NONE};
use crate::util::human_date;
use anyhow::Result;
use std::path::Path;

fn pulled_in_by(lock: &Lock, node: &str) -> String {
    let parents = lock.parents_of(node);
    if parents.is_empty() {
        return NONE.into();
    }
    parents
        .iter()
        .map(|(parent, input)| {
            if *parent == lock.root {
                t::THIS_FLAKE.to_string()
            } else if parent == input {
                fill(t::PARENT, &[("parent", parent)])
            } else {
                fill(t::PARENT_AS, &[("parent", parent), ("input", input)])
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn diamond(o: &mut Vec<String>, lock: &Lock, d: &Dup) {
    let mut rows = Vec::new();
    for (rev, nodes) in &d.revs {
        let short: String = rev.chars().take(7).collect();
        for n in nodes {
            rows.push(fill(
                t::DIAMOND_ROW,
                &[
                    ("rev", &short),
                    ("node", n),
                    ("parents", &pulled_in_by(lock, n)),
                ],
            ));
        }
    }
    o.push(fill(
        t::DIAMOND,
        &[
            ("source", &d.source),
            ("revisions", &d.revs.len().to_string()),
            ("rows", &rows.join("\n")),
        ],
    ));

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
            fixes.push(fill(
                t::FIX_LINE,
                &[("parent", &parent), ("input", &input), ("target", &target)],
            ));
        }
    }
    if fixes.is_empty() {
        return;
    }
    fixes.sort();
    fixes.dedup();
    o.push(fill(
        t::FIX,
        &[("target", &target), ("lines", &fixes.join("\n"))],
    ));
}

fn lock_dates(
    o: &mut Vec<String>,
    out: &Out,
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
                .unwrap_or_else(|| NONE.into()),
        })
        .collect();
    let Some((lo, hi)) = lock.date_span() else {
        return Ok(());
    };

    let svg = chart::timeline(t::DATES_CAPTION, &marks, style);
    out.write(&src.join("inputs-timeline.svg"), &svg)?;

    o.push(t::DATES.into());
    let days = (hi - lo) / 86_400;
    if days > 0 {
        o.push(fill(t::SPAN, &[("days", &days.to_string())]));
    }
    Ok(())
}

pub(super) fn page_inputs(out: &Out, src: &Path, lock: &Lock, style: &D2Style) -> Result<()> {
    let from = out.root.join("inputs.svg");
    if from.exists() {
        let rel = src.join("inputs.svg");
        std::fs::create_dir_all(out.root.join(src))?;
        std::fs::copy(&from, out.root.join(&rel))?;
    }

    let mut rows: Vec<String> = lock
        .inputs()
        .into_iter()
        .map(|(name, locked)| {
            let date = locked
                .last_modified
                .map(human_date)
                .unwrap_or_else(|| NONE.into());
            fill(
                t::ROW,
                &[
                    ("name", name),
                    ("source", &locked.source()),
                    ("rev", &locked.short_rev()),
                    ("date", &date),
                ],
            )
        })
        .collect();
    if rows.is_empty() {
        rows.push(t::EMPTY.into());
    }
    let mut o = vec![
        t::TITLE.to_string(),
        t::INTRO.to_string(),
        fill(t::TABLE, &[("rows", &rows.join("\n"))]),
    ];

    lock_dates(&mut o, out, src, lock, style)?;

    let dups = lock.duplicates();
    let (diamonds, redundant): (Vec<&Dup>, Vec<&Dup>) = dups.iter().partition(|d| d.is_diamond());

    if !diamonds.is_empty() {
        o.push(t::DIAMONDS.into());
        for d in diamonds {
            diamond(&mut o, lock, d);
        }
    }

    if !redundant.is_empty() {
        let rows: Vec<String> = redundant
            .iter()
            .map(|d| {
                let nodes: Vec<String> = d
                    .nodes()
                    .iter()
                    .map(|n| fill(t::NODE, &[("node", n)]))
                    .collect();
                fill(
                    t::REDUNDANT_ROW,
                    &[("source", &d.source), ("nodes", &nodes.join(", "))],
                )
            })
            .collect();
        o.push(fill(t::REDUNDANT, &[("rows", &rows.join("\n"))]));
    }

    page(out, &src.join("inputs.md"), &o)
}
