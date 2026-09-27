use super::{mib, one_line, options, paint, Band, Style};
use crate::conf::palette::chart::TILE_INK;
use crate::text::chart as t;
use anyhow::Result;
use charts_rs::TreemapChart;
use serde_json::json;

const TREE_H: f32 = 400.0;

pub struct Tile {
    pub label: String,
    pub value: u64,
    pub band: Band,
}

pub fn treemap(tiles: &[Tile], style: &Style) -> Result<String> {
    let data: Vec<_> = tiles
        .iter()
        .filter(|t| t.value > 0)
        .map(|t| json!({"name": t.label, "value": mib(t.value), "color": style.color(&t.band.color())}))
        .collect();
    let extra = json!({
        "series_data": data,
        "legend_show": false,
        "series_label_font_color": style.color(&TILE_INK),
        "series_label_formatter": t::MIB,
        "margin": {"left": 0.0, "top": 0.0, "right": 0.0, "bottom": 0.0},
    });
    let mut chart = TreemapChart::from_json(&options(style, TREE_H, extra))?;
    paint(&mut chart, style);
    one_line(&chart.svg()?)
}
