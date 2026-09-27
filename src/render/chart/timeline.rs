use super::{bar_axis, height, options, paint, Style};
use crate::conf::palette::chart as paint_color;
use crate::human::DAY;
use crate::text::chart as t;
use anyhow::Result;
use charts_rs::HorizontalBarChart;
use serde_json::json;

pub struct Mark {
    pub label: String,
    pub at: Option<i64>,
    pub direct: bool,
    pub note: String,
}

pub fn timeline(marks: &[Mark], style: &Style) -> Result<String> {
    let mut order: Vec<&Mark> = marks.iter().collect();
    order.sort_by_key(|m| (m.at.is_none(), m.at, m.label.clone()));
    order.reverse();
    let newest = marks.iter().filter_map(|m| m.at).max().unwrap_or(0);
    let kinds = [
        (true, t::DIRECT, paint_color::MARK),
        (false, t::TRANSITIVE, paint_color::MUTED),
    ];
    let present: Vec<_> = kinds
        .iter()
        .filter(|(d, _, _)| order.iter().any(|m| m.at.is_some() && m.direct == *d))
        .collect();
    let series: Vec<_> = present
        .iter()
        .map(|(direct, name, _)| {
            let data: Vec<Option<f32>> = order
                .iter()
                .map(|m| m.at.filter(|_| m.direct == *direct))
                .map(|at| at.map(|at| ((newest - at) / DAY) as f32))
                .collect();
            json!({"name": name, "data": data, "stack": "age", "label_show": true})
        })
        .collect();
    let colors: Vec<&str> = present.iter().map(|(_, _, c)| style.color(c)).collect();
    let labels = order.iter().map(|m| t::row(&m.label, &m.note)).collect();
    let mut extra = bar_axis(style, labels);
    extra["series_list"] = series.into();
    extra["series_colors"] = colors.into();
    extra["legend_show"] = (present.len() > 1).into();
    extra["series_label_formatter"] = t::DAYS.into();
    let mut chart = HorizontalBarChart::from_json(&options(style, height(order.len()), extra))?;
    paint(&mut chart, style);
    Ok(chart.svg()?)
}
