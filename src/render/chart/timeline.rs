//! The lock timeline: every input placed by the date it was locked at.
//!
//! `lastModified` is a fixed integer stored in `flake.lock`, so this draws the
//! *spread* of a supply chain and never a judgement about now. "Overdue" would
//! need a clock, which would make two builds of the same input disagree and
//! cost mode B its reproducibility — the reason the atlas plate it comes from
//! (6.2, input staleness) is rendered de-clocked or not at all.
//!
//! Direct and transitive inputs are coloured apart because they move under
//! different actions: `nix flake update` bumps what this flake declares, while
//! everything else only moves when whatever pulled it in does. A lone old row
//! is a different job depending on which it is.

use super::{color, gutter, legend, rect, svg_open, text, D2Style, Key, LEGEND_H, PAD, W};
use std::cmp::Ordering;

const ROW_H: u64 = 20;
const TICK_W: u64 = 3;
const TICK_H: u64 = 11;

/// Hue-matched to the diagrams' mesh blue, but under a chart-only name: an
/// entry in `PALETTE` is written into every diagram's `vars` block and would
/// churn every snapshot here and in every consumer's committed docs.
const DIRECT: Key = Key {
    color: ("chartMark", "#4a76c4", "#7fa7e8"),
    label: "declared by this flake",
};
const TRANSITIVE: Key = Key {
    color: ("chartMuted", "#777777", "#8b949e"),
    label: "pulled in by an input",
};

/// One input on the timeline.
pub struct Mark {
    pub label: String,
    /// `lastModified` out of the lock. `None` for an input that carries no
    /// date — a `path:` input, typically. The row keeps its name and draws no
    /// tick, so an input that cannot be placed is visibly present rather than
    /// quietly dropped.
    pub at: Option<i64>,
    /// Declared by the root flake itself, rather than reached through another
    /// input.
    pub direct: bool,
    /// Right-hand annotation: the formatted date, or why there is none.
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

/// A dot plot of dates: one row per input, ticks on a shared axis running from
/// the oldest lock date to the newest.
///
/// Sorts internally, oldest first, so the caller's order cannot break the
/// staircase — and so the first and last rows' own notes label the ends of the
/// axis, which is why the chart needs no separate scale.
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

    // One kind alone draws no distinction, so it earns no key — the same rule
    // the bands follow when a single measured host leaves nothing to compare.
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
        // Both gutters are right-aligned inward, so labels sit against the
        // plot and nothing rides the canvas edge, whatever the name lengths.
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
            // The axis is drawn per row rather than once, so an undated input
            // gets no track: it has no place on this scale and should not look
            // as though it does.
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
    use crate::render::out::MD_MARKER;

    fn mark(label: &str, at: Option<i64>, direct: bool) -> Mark {
        Mark {
            label: label.into(),
            at,
            direct,
            note: at.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
        }
    }

    /// x of every tick, in the order they were drawn: the ticks are the only
    /// rects as narrow as `TICK_W`, since the tracks span the plot.
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

    /// Everything between the `>` and `<` of each `<text>`, in draw order.
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
        // The first tick opens the plot and the last one closes it, so the two
        // notes beside them label the scale and no separate axis is drawn.
        let plot_start = t[0];
        assert!(t[1] > plot_start && t[2] > t[1], "{t:?}");
        assert_eq!(t[1] - plot_start, (t[2] - plot_start) / 2, "{t:?}");
    }

    #[test]
    fn a_single_date_puts_every_tick_at_the_start() {
        // Nothing separates the inputs, so nothing should be implied by
        // position — a divide by the zero span would be the bug here.
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
        // Undated rows sort last, so the dated one keeps the top.
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
        assert!(mixed.contains("declared by this flake"), "{mixed}");
        assert!(mixed.contains("pulled in by an input"), "{mixed}");

        let all_direct = timeline(
            "t",
            &[mark("a", Some(1), true), mark("b", Some(2), true)],
            &D2Style::default(),
        );
        assert!(
            !all_direct.contains("declared by this flake"),
            "{all_direct}"
        );
    }

    #[test]
    fn labels_are_xml_escaped_and_the_marker_leads() {
        let svg = timeline("a & b", &[mark("<in>", Some(1), true)], &D2Style::default());
        assert!(svg.starts_with(MD_MARKER), "{svg}");
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
