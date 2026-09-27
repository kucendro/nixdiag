mod bar;
mod canvas;
mod timeline;
mod treemap;

pub use bar::{bars, Row};
pub use timeline::{timeline, Mark};
pub use treemap::{treemap, Tile};

use super::style::Style;
use crate::conf::palette::{chart as paint, Color};
use crate::text::chart as t;
use canvas::{top, Canvas, Frame};

const W: u64 = 720;
const PAD: u64 = 8;
const CH: u64 = 7;
const LEGEND_H: u64 = 24;
const SWATCH: u64 = 10;
const INSET: u64 = 4;
const LABEL_PX: u64 = 13;
const NOTE_PX: u64 = 12;

#[derive(PartialEq, Eq)]
struct Key {
    color: Color,
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

    fn color(self) -> Color {
        match self {
            Band::Solid | Band::Shared => paint::SHARED,
            Band::Partial => paint::PARTIAL,
            Band::Unique => paint::UNIQUE,
            Band::Rest => paint::MUTED,
        }
    }
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
