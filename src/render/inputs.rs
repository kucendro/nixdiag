//! Flake input graph: the documented flake's own supply chain.
//!
//! Everything here comes out of `flake.lock`, so the diagram is a pure
//! function of a file already in the repo — no eval, no realisation.
//!
//! Only nodes caught by duplicate detection carry their revision in the
//! label. A rev on every box is noise; a rev on the two boxes that disagree
//! is the whole point of the picture.

use super::d2::{write_and_render, D2Style, D2_HEADER};
use super::out::Out;
use crate::source::flakelock::Lock;
use crate::util::sanitize;
use anyhow::Result;
use std::collections::BTreeSet;

pub fn generate(lock: &Lock, out: &mut Out, render_svg: bool, style: &D2Style) -> Result<()> {
    let dups = lock.duplicates();
    let flagged: BTreeSet<&str> = dups
        .iter()
        .filter(|d| d.is_diamond())
        .flat_map(|d| d.nodes())
        .collect();

    let mut o: Vec<String> = D2_HEADER.iter().map(|s| s.to_string()).collect();
    o.extend(super::d2::vars_block(style));
    o.push("direction: right".into());
    o.push(String::new());

    o.push(format!(
        "{}: \"this flake\" {{ shape: cloud; style.fill: ${{hostCloud}}; style.bold: true }}",
        sanitize(&lock.root)
    ));
    for (name, locked) in lock.inputs() {
        let label = if flagged.contains(name.as_str()) {
            format!("{name} {}", locked.short_rev())
        } else {
            name.clone()
        };
        let stroke = if flagged.contains(name.as_str()) {
            "style.stroke: ${public}; style.stroke-width: 2"
        } else {
            "style.stroke: ${baseStroke}"
        };
        o.push(format!(
            "{}: \"{label}\" {{ style.fill: ${{baseFill}}; {stroke} }}",
            sanitize(name)
        ));
    }

    let mut direct: Vec<String> = Vec::new();
    let mut follows: Vec<String> = Vec::new();
    for (parent, input, child, is_follows) in lock.edges() {
        // The input name is only worth showing when it differs from the node
        // it resolves to (`utils` pointing at `flake-utils`).
        let label = if input == child {
            String::new()
        } else {
            format!(": \"{input}\"")
        };
        let (a, b) = (sanitize(&parent), sanitize(&child));
        if is_follows {
            follows.push(format!(
                "{a} -> {b}{label} {{ style.stroke: ${{mesh}}; style.stroke-dash: 3 }}"
            ));
        } else {
            direct.push(format!("{a} -> {b}{label}"));
        }
    }

    o.push(String::new());
    o.push("# direct inputs".into());
    o.extend(direct);
    if !follows.is_empty() {
        o.push(String::new());
        o.push("# follows (deduplication)".into());
        o.extend(follows);
    }

    write_and_render(out, "inputs", &o, render_svg, style)
}
