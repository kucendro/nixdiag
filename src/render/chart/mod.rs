//! Charts nixdiag draws itself, as plain SVG.
//!
//! Not another renderer dependency. d2+elk has no area or length channel, so
//! quantitative plates cannot be drawn with it at all, and every external
//! chart tool would be a permanent promise to track an upstream — the
//! argument that killed the adapter layer in v2. Emitting the SVG here costs
//! a page of integer arithmetic and buys two things d2 cannot:
//!
//! * no binary on PATH, so the picture exists wherever `render` runs, and
//! * byte-determinism, so the file is written as `Auto` and the drift gate
//!   covers it. Every d2-produced SVG is excluded from `check` precisely
//!   because its bytes move with the d2 version.
//!
//! Colors resolve through `d2::color`, but the names used here are
//! deliberately absent from `PALETTE`: `vars_block` writes the whole palette
//! into every diagram, so a new entry would churn every snapshot in this repo
//! and every consumer's committed docs. `--color chartUnique=#hex` still
//! works, because the override table is consulted before the palette.
//!
//! This file is the façade and the shared vocabulary — the band colours, the
//! canvas geometry, and the handful of primitives that put a rectangle or a
//! string on it. One chart per submodule below, each private, so the crate
//! only ever sees the `pub use` list.

mod bar;
mod timeline;
mod treemap;

pub use bar::{bars, Row};
pub use timeline::{timeline, Mark};
pub use treemap::{treemap, Tile};

use super::d2::{color, D2Style};
use super::out::MD_MARKER;

/// Canvas width. mdBook's content column is a little under 750px, so a
/// fixed-width chart lands unscaled at the default theme and shrinks
/// proportionally on anything narrower.
const W: u64 = 720;
const PAD: u64 = 8;
/// Approximate advance width of the label font at 13px. Only used to size
/// gutters and decide whether a label fits, so being a little generous costs
/// whitespace rather than clipping.
const CH: u64 = 7;
const LEGEND_H: u64 = 24;
const SWATCH: u64 = 10;

/// One entry in a chart's key: the colour override name with its light and
/// dark defaults, and what that colour means. A chart that colours something
/// other than closure bands builds these itself.
#[derive(PartialEq, Eq)]
struct Key {
    color: (&'static str, &'static str, &'static str),
    label: &'static str,
}

/// How widely the paths in one segment are held across the measured fleet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// Fewer than two hosts measured, so there was nothing to compare.
    Solid,
    Shared,
    Partial,
    Unique,
    /// The treemap's long tail, folded into one tile.
    Rest,
}

impl Band {
    fn legend(self) -> &'static str {
        match self {
            Band::Solid => "closure",
            Band::Shared => "shared by every host",
            Band::Partial => "shared by some",
            Band::Unique => "unique to this host",
            Band::Rest => "everything smaller",
        }
    }

    fn key(self) -> Key {
        Key {
            color: self.color(),
            label: self.legend(),
        }
    }

    /// (override name, light, dark). Hues match the diagrams' semantic
    /// colors, but under their own names — see the module docs.
    fn color(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Band::Solid | Band::Shared => ("chartShared", "#4a76c4", "#7fa7e8"),
            Band::Partial => ("chartPartial", "#c47a29", "#d9995a"),
            Band::Unique => ("chartUnique", "#27893f", "#2ecc71"),
            Band::Rest => ("chartMuted", "#777777", "#8b949e"),
        }
    }
}

fn xml_escape(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            _ => o.push(c),
        }
    }
    o
}

/// Gutter wide enough for the longest of these strings.
fn gutter<'a>(strings: impl Iterator<Item = &'a str>) -> u64 {
    let longest = strings
        .map(str::chars)
        .map(Iterator::count)
        .max()
        .unwrap_or(0);
    CH * longest as u64 + 12
}

fn rect(o: &mut String, x: u64, y: u64, w: u64, h: u64, fill: &str) {
    o.push_str(&format!(
        "  <rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill=\"{fill}\"/>\n"
    ));
}

/// Document header: the AUTO marker (an XML comment, so the Markdown one
/// serves verbatim), the root element, and the optional `--background` fill.
fn svg_open(caption: &str, h: u64, style: &D2Style) -> String {
    let mut o = format!(
        "{MD_MARKER}\n\
         <svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {W} {h}\" \
         width=\"{W}\" height=\"{h}\" role=\"img\" \
         font-family=\"ui-sans-serif, system-ui, sans-serif\">\n\
         \x20 <title>{}</title>\n",
        xml_escape(caption)
    );
    if let Some(bg) = &style.background {
        rect(&mut o, 0, 0, W, h, bg);
    }
    o
}

fn text(o: &mut String, x: u64, y: u64, size: u64, fill: &str, end: bool, s: &str) {
    let anchor = if end { " text-anchor=\"end\"" } else { "" };
    o.push_str(&format!(
        "  <text x=\"{x}\" y=\"{y}\" font-size=\"{size}\" fill=\"{fill}\"{anchor}>{}</text>\n",
        xml_escape(s)
    ));
}

/// The distinct bands present, in first-seen order.
///
/// A lone `Solid` earns no key: it is the "nothing to compare" case, so a
/// legend would name a distinction the picture does not draw.
fn legend_bands(bands: impl Iterator<Item = Band>) -> Vec<Key> {
    let mut o: Vec<Band> = Vec::new();
    for b in bands {
        if !o.contains(&b) {
            o.push(b);
        }
    }
    if o == [Band::Solid] {
        o.clear();
    }
    o.into_iter().map(Band::key).collect()
}

/// Swatch-and-label key along the top, running right from `x`.
fn legend(o: &mut String, keys: &[Key], mut x: u64, style: &D2Style) {
    let muted = color(style, "chartMuted", ("#777777", "#8b949e"));
    for key in keys {
        let (name, light, dark) = key.color;
        rect(
            o,
            x,
            PAD + 2,
            SWATCH,
            SWATCH,
            color(style, name, (light, dark)),
        );
        text(o, x + SWATCH + 5, PAD + 11, 12, muted, false, key.label);
        x += SWATCH + 5 + CH * key.label.len() as u64 + 14;
    }
}
