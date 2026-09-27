use super::d2::{preamble, write_and_render};
use super::out::Out;
use super::style::Style;
use crate::conf::files::diagram;
use crate::source::flakelock::Lock;
use crate::text::d2::inputs as t;
use crate::text::fill;
use crate::util::sanitize;
use anyhow::Result;
use std::collections::BTreeSet;

pub fn generate(lock: &Lock, out: &Out, render_svg: bool, style: &Style) -> Result<()> {
    let dups = lock.duplicates();
    let flagged: BTreeSet<&str> = dups
        .iter()
        .filter(|d| d.is_diamond())
        .flat_map(|d| d.nodes())
        .collect();

    let mut o = preamble(style);
    o.push(String::new());
    o.push(fill(t::ROOT, &[("id", &sanitize(&lock.root))]));
    for (name, locked) in lock.inputs() {
        let (label, stroke) = if flagged.contains(name.as_str()) {
            (
                fill(
                    t::FLAGGED_LABEL,
                    &[("name", name), ("rev", &locked.short_rev())],
                ),
                t::FLAGGED_STROKE,
            )
        } else {
            (name.clone(), t::STROKE)
        };
        o.push(fill(
            t::INPUT,
            &[
                ("id", &sanitize(name)),
                ("label", &label),
                ("stroke", stroke),
            ],
        ));
    }

    let mut direct: Vec<String> = Vec::new();
    let mut follows: Vec<String> = Vec::new();
    for e in lock.edges() {
        let label = if e.input == e.child {
            String::new()
        } else {
            fill(t::EDGE_LABEL, &[("input", &e.input)])
        };
        let (from, to) = (sanitize(&e.parent), sanitize(&e.child));
        let vars = [("from", from.as_str()), ("to", &to), ("label", &label)];
        if e.follows {
            follows.push(fill(t::FOLLOWS, &vars));
        } else {
            direct.push(fill(t::EDGE, &vars));
        }
    }

    o.push(String::new());
    o.push(t::DIRECT_EDGES.into());
    o.extend(direct);
    if !follows.is_empty() {
        o.push(String::new());
        o.push(t::FOLLOWS_EDGES.into());
        o.extend(follows);
    }

    write_and_render(out, diagram::INPUTS, &o, render_svg, style)
}
