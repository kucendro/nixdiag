use super::super::chart::{self, Mark};
use super::{code, codes, table, Wiki};
use crate::conf::files::{chart as svg, diagram, page};
use crate::source::flakelock::{short, Dup, Lock, Locked};
use crate::text::fill;
use crate::text::wiki::{inputs as t, NONE};
use crate::util::{human_date, DAY};
use anyhow::Result;

fn date(l: &Locked) -> String {
    l.last_modified.map_or(NONE.into(), human_date)
}

fn pulled_in_by(lock: &Lock, node: &str) -> String {
    let parents = lock.parents_of(node);
    if parents.is_empty() {
        return NONE.into();
    }
    parents
        .iter()
        .map(|e| {
            if e.parent == lock.root {
                t::THIS_FLAKE.to_string()
            } else if e.parent == e.input {
                code(&e.parent)
            } else {
                fill(t::PARENT_AS, &[("parent", &e.parent), ("input", &e.input)])
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn diamond(o: &mut Vec<String>, lock: &Lock, d: &Dup) {
    let mut rows = Vec::new();
    for r in &d.revs {
        for n in &r.nodes {
            rows.push(fill(
                t::DIAMOND_ROW,
                &[
                    ("rev", &short(&r.rev)),
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
        ],
    ));
    o.push(table(&t::DIAMOND_HEAD, &rows));

    let Some(target) = lock.root_input_for(&d.identity) else {
        return;
    };
    let mut fixes: Vec<String> = Vec::new();
    for n in d.nodes() {
        if n == target {
            continue;
        }
        for e in lock.parents_of(n) {
            if e.parent == lock.root {
                continue;
            }
            fixes.push(fill(
                t::FIX_LINE,
                &[
                    ("parent", &e.parent),
                    ("input", &e.input),
                    ("target", &target),
                ],
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

fn lock_dates(o: &mut Vec<String>, w: &Wiki, lock: &Lock) -> Result<()> {
    let roots = lock.root_inputs();
    let marks: Vec<Mark> = lock
        .inputs()
        .into_iter()
        .map(|(name, locked)| Mark {
            label: name.clone(),
            at: locked.last_modified,
            direct: roots.contains(name.as_str()),
            note: date(locked),
        })
        .collect();
    let Some((lo, hi)) = lock.date_span() else {
        return Ok(());
    };

    w.src.write(
        svg::TIMELINE,
        &chart::timeline(t::DATES_CAPTION, &marks, w.style),
    )?;

    o.push(t::DATES.into());
    let days = (hi - lo) / DAY;
    if days > 0 {
        o.push(fill(t::SPAN, &[("days", &days.to_string())]));
    }
    Ok(())
}

pub(super) fn page_inputs(w: &Wiki, lock: &Lock) -> Result<()> {
    w.src.mirror(w.out, &format!("{}.svg", diagram::INPUTS))?;

    let rows: Vec<String> = lock
        .inputs()
        .into_iter()
        .map(|(name, locked)| {
            fill(
                t::ROW,
                &[
                    ("name", name),
                    ("source", &locked.source()),
                    ("rev", &locked.short_rev()),
                    ("date", &date(locked)),
                ],
            )
        })
        .collect();
    let mut o = vec![
        t::TITLE.to_string(),
        t::INTRO.to_string(),
        table(&t::HEAD, &rows),
    ];

    lock_dates(&mut o, w, lock)?;

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
                fill(
                    t::REDUNDANT_ROW,
                    &[("source", &d.source), ("nodes", &codes(d.nodes(), ", "))],
                )
            })
            .collect();
        o.push(fill(t::REDUNDANT, &[("rows", &rows.join("\n"))]));
    }

    w.page(page::INPUTS, &o)
}
