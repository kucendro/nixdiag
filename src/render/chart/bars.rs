use super::{bar_axis, height, mib, options, paint, Band, Style};
use crate::text::chart as t;
use anyhow::Result;
use charts_rs::HorizontalBarChart;
use serde_json::json;

pub struct Row {
    pub label: String,
    pub bands: Vec<(Band, u64)>,
    pub note: String,
}

impl Row {
    fn value(&self, band: Band) -> Option<f32> {
        let v: u64 = self
            .bands
            .iter()
            .filter(|(b, _)| *b == band)
            .map(|(_, v)| v)
            .sum();
        (v > 0).then(|| mib(v))
    }
}

pub fn bars(rows: &[Row], style: &Style) -> Result<String> {
    let rows: Vec<&Row> = rows.iter().rev().collect();
    let mut bands: Vec<Band> = Vec::new();
    for (b, v) in rows.iter().flat_map(|r| &r.bands) {
        if *v > 0 && !bands.contains(b) {
            bands.push(*b);
        }
    }
    let series: Vec<_> = bands
        .iter()
        .map(|b| {
            let data: Vec<Option<f32>> = rows.iter().map(|r| r.value(*b)).collect();
            json!({"name": b.legend(), "data": data, "stack": "size"})
        })
        .collect();
    let colors: Vec<&str> = bands.iter().map(|b| style.color(&b.color())).collect();
    let labels = rows.iter().map(|r| t::row(&r.label, &r.note)).collect();
    let mut extra = bar_axis(style, labels);
    extra["series_list"] = series.into();
    extra["series_colors"] = colors.into();
    extra["legend_show"] = (bands != [Band::Solid]).into();
    extra["y_axis_configs"][0]["axis_formatter"] = t::MIB.into();
    let mut chart = HorizontalBarChart::from_json(&options(style, height(rows.len()), extra))?;
    paint(&mut chart, style);
    Ok(chart.svg()?)
}
