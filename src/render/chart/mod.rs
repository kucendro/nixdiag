mod bars;
mod timeline;
mod treemap;

pub use bars::{bars, Row};
pub use timeline::{timeline, Mark};
pub use treemap::{treemap, Tile};

use super::style::Style;
use super::svg::one_line;
use crate::conf::palette::{chart as paint, Color};
use crate::text::chart as t;
use charts_rs::ChartBase;
use serde_json::{json, Value};

const FONT: &str = "Roboto, Arial, sans-serif";
const W: f32 = 720.0;
const ROW_H: f32 = 30.0;
const CHROME_H: f32 = 64.0;
const MIB: f32 = 1_048_576.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    fn color(self) -> Color {
        match self {
            Band::Solid | Band::Shared => paint::SHARED,
            Band::Partial => paint::PARTIAL,
            Band::Unique => paint::UNIQUE,
            Band::Rest => paint::MUTED,
        }
    }
}

fn mib(bytes: u64) -> f32 {
    bytes as f32 / MIB
}

fn height(rows: usize) -> f32 {
    CHROME_H + ROW_H * rows as f32
}

fn options(style: &Style, height: f32, extra: Value) -> String {
    let (ink, muted, grid) = (
        style.color(&paint::INK),
        style.color(&paint::MUTED),
        style.color(&paint::TRACK),
    );
    let mut o = json!({
        "theme": style.theme.chart(),
        "font_family": FONT,
        "width": W,
        "height": height,
        "margin": {"left": 8.0, "top": 8.0, "right": 72.0, "bottom": 8.0},
        "legend_font_color": muted,
        "legend_category": "rect",
        "legend_align": "left",
        "x_axis_font_color": ink,
        "x_axis_stroke_color": grid,
        "grid_stroke_color": grid,
        "series_label_font_color": ink,
    });
    if let (Some(o), Some(extra)) = (o.as_object_mut(), extra.as_object()) {
        o.extend(extra.clone());
    }
    o.to_string()
}

fn bar_axis(style: &Style, labels: Vec<String>) -> Value {
    json!({
        "series_label_position": "right",
        "x_axis_data": labels,
        "x_axis_name_gap": 12.0,
        "y_axis_configs": [{
            "axis_font_color": style.color(&paint::MUTED),
            "axis_stroke_color": style.color(&paint::TRACK),
        }],
    })
}

fn paint(chart: &mut ChartBase, style: &Style) {
    chart.background_color = style
        .background
        .as_deref()
        .map_or(charts_rs::Color::transparent(), Into::into);
}
