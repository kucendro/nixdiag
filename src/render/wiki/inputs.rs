use super::super::chart::{self, Mark};
use super::{code, codes, table, Page, Wiki};
use crate::conf::files::{chart as svg, diagram, page};
use crate::human::{Date, DAY};
use crate::source::flakelock::{short, Dup, Lock, Locked};
use crate::text::wiki::{inputs as t, NONE};
use anyhow::Result;

fn date(l: &Locked) -> String {
    l.last_modified.map_or(NONE.into(), |s| Date(s).to_string())
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
                t::parent_as(&e.parent, &e.input)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn diamond(o: &mut Vec<String>, lock: &Lock, d: &Dup) {
    let rows = d.revs.iter().flat_map(|r| {
        let row = |n: &String| [code(short(&r.rev)), code(n), pulled_in_by(lock, n)];
        r.nodes.iter().map(row)
    });
    o.push(t::diamond(&d.source, d.revs.len()));
    o.push(table(t::DIAMOND_HEAD, rows));

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
            fixes.push(t::fix_line(&e.parent, &e.input, &target));
        }
    }
    if fixes.is_empty() {
        return;
    }
    fixes.sort();
    fixes.dedup();
    o.push(t::fix(&target, &fixes.join("\n")));
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

    w.src
        .write(svg::TIMELINE, &chart::timeline(&marks, w.style)?)?;

    o.push(t::DATES.into());
    let days = (hi - lo) / DAY;
    if days > 0 {
        o.push(t::span(days));
    }
    Ok(())
}

pub(super) struct Inputs;

impl Page for Inputs {
    fn file(&self) -> &'static str {
        page::INPUTS
    }

    fn title(&self) -> &'static str {
        t::TITLE
    }

    fn body(&self, w: &Wiki) -> Result<Option<Vec<String>>> {
        w.lock.map(|lock| body(w, lock)).transpose()
    }
}

fn body(w: &Wiki, lock: &Lock) -> Result<Vec<String>> {
    w.src.mirror(w.out, &format!("{}.svg", diagram::INPUTS))?;

    let rows = lock
        .inputs()
        .into_iter()
        .map(|(name, l)| [code(name), code(l.source()), code(l.short_rev()), date(l)]);
    let mut o = vec![t::INTRO.to_string(), table(t::HEAD, rows)];

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
        let rows = redundant
            .iter()
            .map(|d| t::redundant(&d.source, &codes(d.nodes(), ", ")));
        o.push(t::REDUNDANT.into());
        o.push(rows.collect::<Vec<_>>().join("\n"));
    }
    Ok(o)
}
