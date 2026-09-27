use super::dot::{id, Dot, Paint};
use crate::conf::files::diagram;
use crate::conf::palette::diagram as p;
use crate::source::flakelock::Lock;
use crate::text::dot::inputs as t;
use anyhow::Result;
use dot_writer::{Attributes, Style};
use std::collections::BTreeSet;

pub fn generate(lock: &Lock, dot: &Dot) -> Result<()> {
    let dups = lock.duplicates();
    let flagged: BTreeSet<&str> = dups
        .iter()
        .filter(|d| d.is_diamond())
        .flat_map(|d| d.nodes())
        .collect();

    dot.render(diagram::INPUTS, |g| {
        g.node_named(id(&lock.root))
            .bold(t::ROOT)
            .fill(dot.color(&p::HOST_CLOUD))
            .stroke(dot.color(&p::HOST_STROKE));
        for (name, locked) in lock.inputs() {
            let mut n = g.node_named(id(name));
            if flagged.contains(name.as_str()) {
                n.text(&[&t::flagged(name, &locked.short_rev())])
                    .stroke(dot.color(&p::PUBLIC))
                    .set_pen_width(2.5);
            } else {
                n.text(&[name]);
            }
        }
        for e in lock.edges() {
            let mut edge = g.edge(id(&e.parent), id(&e.child)).attributes();
            if e.input != e.child {
                edge.text(&[&e.input]);
            }
            if e.follows {
                edge.stroke(dot.color(&p::MESH)).set_style(Style::Dashed);
            }
        }
    })
}
