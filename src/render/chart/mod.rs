mod bar;
mod timeline;
mod treemap;

pub use bar::{bars, Row};
pub use timeline::{timeline, Mark};
pub use treemap::{treemap, Tile};

use super::d2::{color, D2Style};
use super::out::MD_MARKER;
use crate::text::chart as t;

const W: u64 = 720;
const PAD: u64 = 8;
const CH: u64 = 7;
const LEGEND_H: u64 = 24;
const SWATCH: u64 = 10;

#[derive(PartialEq, Eq)]
struct Key {
    color: (&'static str, &'static str, &'static str),
    label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Solid,
    Shared,
    Partial,
    Unique,
    Rest,
}

impl Band {
    fn legend(self) -> &'static str {
        match self {
            Band::Solid => t::CLOSURE,
            Band::Shared => t::SHARED,
            Band::Partial => t::PARTIAL,
            Band::Unique => t::UNIQUE,
            Band::Rest => t::REST,
        }
    }

    fn key(self) -> Key {
        Key {
            color: self.color(),
            label: self.legend(),
        }
    }

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
