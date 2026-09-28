use super::d2::{Class, Diagram, Doc};
use crate::conf::files::diagram;
use crate::source::flakelock::Lock;
use crate::text::d2::inputs as t;
use std::collections::BTreeSet;

pub struct Inputs<'a>(pub &'a Lock);

impl Diagram for Inputs<'_> {
    fn stem(&self) -> String {
        diagram::INPUTS.into()
    }

    fn draw(&self, doc: &mut Doc) {
        let lock = self.0;
        let dups = lock.duplicates();
        let flagged: BTreeSet<&str> = dups
            .iter()
            .filter(|d| d.is_diamond())
            .flat_map(|d| d.nodes())
            .collect();
        doc.shape(&lock.root, t::ROOT, Class::Root);
        for (name, locked) in lock.inputs() {
            if flagged.contains(name.as_str()) {
                doc.shape(name, &t::flagged(name, &locked.short_rev()), Class::Flagged);
            } else {
                doc.shape(name, name, Class::Node);
            }
        }
        for e in lock.edges() {
            let label = (e.input != e.child).then_some(e.input.as_str());
            let class = if e.follows {
                Class::Follows
            } else {
                Class::Arrow
            };
            doc.edge(&[&e.parent], &[&e.child], label, class);
        }
    }
}
