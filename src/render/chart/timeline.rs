use super::{color, gutter, legend, rect, svg_open, text, D2Style, Key, LEGEND_H, PAD, W};
use crate::text::chart as t;
use std::cmp::Ordering;

const ROW_H: u64 = 20;
const TICK_W: u64 = 3;
const TICK_H: u64 = 11;

const DIRECT: Key = Key {
    color: ("chartMark", "#4a76c4", "#7fa7e8"),
    label: t::DIRECT,
};
const TRANSITIVE: Key = Key {
    color: ("chartMuted", "#777777", "#8b949e"),
    label: t::TRANSITIVE,
};

pub struct Mark {
    pub label: String,
    pub at: Option<i64>,
    pub direct: bool,
    pub note: String,
}

impl Mark {
    fn key(&self) -> Key {
        if self.direct {
            DIRECT
        } else {
            TRANSITIVE
        }
    }
}

pub fn timeline(caption: &str, marks: &[Mark], style: &D2Style) -> String {
    let ink = color(style, "chartInk", ("#333333", "#c9d1d9"));
    let muted = color(style, "chartMuted", ("#777777", "#8b949e"));
    let track = color(style, "chartTrack", ("#ebebeb", "#2a2a2e"));

    let mut order: Vec<&Mark> = marks.iter().collect();
    order.sort_by(|a, b| match (a.at, b.at) {
        (Some(x), Some(y)) => x.cmp(&y).then(a.label.cmp(&b.label)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.label.cmp(&b.label),
    });

    let label_w = gutter(order.iter().map(|m| m.label.as_str()));
    let note_w = gutter(order.iter().map(|m| m.note.as_str()));
    let plot_w = W.saturating_sub(label_w + note_w + PAD).max(1);

    let mut keys: Vec<Key> = Vec::new();
    for direct in [true, false] {
        if order.iter().any(|m| m.at.is_some() && m.direct == direct) {
            keys.push(if direct { DIRECT } else { TRANSITIVE });
        }
    }
    if keys.len() < 2 {
        keys.clear();
    }

    let top = if keys.is_empty() { PAD } else { PAD + LEGEND_H };
    let h = top + ROW_H * order.len() as u64 + PAD;
    let lo = order.iter().filter_map(|m| m.at).min();
    let hi = order.iter().filter_map(|m| m.at).max();

    let mut o = svg_open(caption, h, style);
    legend(&mut o, &keys, label_w, style);

    for (i, m) in order.iter().enumerate() {
        let cy = top + ROW_H * i as u64 + ROW_H / 2;
        text(
            &mut o,
            label_w - PAD,
            cy + 4,
            13,
            if m.at.is_some() { ink } else { muted },
            true,
            &m.label,
        );
        if let (Some(at), Some(lo), Some(hi)) = (m.at, lo, hi) {
            rect(&mut o, label_w, cy, plot_w, 1, track);
            let span = hi - lo;
            let x = if span > 0 {
                (at - lo) as u64 * plot_w.saturating_sub(TICK_W) / span as u64
            } else {
                0
            };
            let (name, light, dark) = m.key().color;
            let fill = color(style, name, (light, dark));
            rect(&mut o, label_w + x, cy - TICK_H / 2, TICK_W, TICK_H, fill);
        }
        text(
            &mut o,
            W - 4,
            cy + 4,
            12,
            if m.at.is_some() { ink } else { muted },
            true,
            &m.note,
        );
    }

    o.push_str("</svg>");
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mark(label: &str, at: Option<i64>, direct: bool) -> Mark {
        Mark {
            label: label.into(),
            at,
            direct,
            note: at.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
        }
    }

    fn ticks(svg: &str) -> Vec<u64> {
        let attr = |l: &str, k: &str| -> Option<u64> {
            l.split(&format!("{k}=\""))
                .nth(1)?
                .split('"')
                .next()?
                .parse()
                .ok()
        };
        svg.lines()
            .filter(|l| l.contains("<rect"))
            .filter(|l| attr(l, "width") == Some(TICK_W))
            .filter_map(|l| attr(l, "x"))
            .collect()
    }

    fn labels(svg: &str) -> Vec<String> {
        svg.lines()
            .filter(|l| l.contains("<text"))
            .filter_map(|l| l.rsplit_once('>')?.0.rsplit_once('>'))
            .map(|(_, s)| s.trim_end_matches("</text").to_string())
            .collect()
    }

    #[test]
    fn rows_run_oldest_first_whatever_order_they_arrive_in() {
        let svg = timeline(
            "t",
            &[
                mark("new", Some(300), true),
                mark("old", Some(100), true),
                mark("mid", Some(200), true),
            ],
            &D2Style::default(),
        );
        let names: Vec<String> = labels(&svg)
            .into_iter()
            .filter(|s| ["old", "mid", "new"].contains(&s.as_str()))
            .collect();
        assert_eq!(names, ["old", "mid", "new"], "{svg}");
    }

    #[test]
    fn the_ends_of_the_axis_are_the_oldest_and_newest_dates() {
        let svg = timeline(
            "t",
            &[
                mark("old", Some(100), true),
                mark("mid", Some(150), true),
                mark("new", Some(200), true),
            ],
            &D2Style::default(),
        );
        let t = ticks(&svg);
        assert_eq!(t.len(), 3, "{svg}");
        let plot_start = t[0];
        assert!(t[1] > plot_start && t[2] > t[1], "{t:?}");
        assert_eq!(t[1] - plot_start, (t[2] - plot_start) / 2, "{t:?}");
    }

    #[test]
    fn a_single_date_puts_every_tick_at_the_start() {
        let svg = timeline(
            "t",
            &[mark("a", Some(7), true), mark("b", Some(7), true)],
            &D2Style::default(),
        );
        let t = ticks(&svg);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0], t[1], "{t:?}");
    }

    #[test]
    fn an_undated_input_keeps_its_row_and_draws_no_tick() {
        let svg = timeline(
            "t",
            &[mark("dated", Some(1), true), mark("path", None, true)],
            &D2Style::default(),
        );
        assert!(svg.contains(">path<"), "{svg}");
        assert!(svg.contains(">—<"), "{svg}");
        assert_eq!(ticks(&svg).len(), 1, "{svg}");
        let names = labels(&svg);
        assert!(
            names.iter().position(|s| s == "dated") < names.iter().position(|s| s == "path"),
            "{names:?}"
        );
    }

    #[test]
    fn the_legend_appears_only_when_both_kinds_occur() {
        let mixed = timeline(
            "t",
            &[mark("a", Some(1), true), mark("b", Some(2), false)],
            &D2Style::default(),
        );
        assert!(mixed.contains(t::DIRECT), "{mixed}");
        assert!(mixed.contains(t::TRANSITIVE), "{mixed}");

        let all_direct = timeline(
            "t",
            &[mark("a", Some(1), true), mark("b", Some(2), true)],
            &D2Style::default(),
        );
        assert!(!all_direct.contains(t::DIRECT), "{all_direct}");
    }

    #[test]
    fn labels_are_xml_escaped() {
        let svg = timeline("a & b", &[mark("<in>", Some(1), true)], &D2Style::default());
        assert!(svg.starts_with("<svg"), "{svg}");
        assert!(svg.contains("<title>a &amp; b</title>"), "{svg}");
        assert!(svg.contains("&lt;in&gt;"), "{svg}");
    }

    #[test]
    fn a_color_override_reaches_the_marker() {
        let style = D2Style {
            colors: vec![("chartMark".into(), "#ff0000".into())],
            ..D2Style::default()
        };
        let svg = timeline("t", &[mark("a", Some(1), true)], &style);
        assert!(svg.contains("#ff0000"), "{svg}");
    }
}
