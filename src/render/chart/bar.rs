//! The fleet bar: one horizontal stacked bar per entry.
//!
//! What a host costs *beyond* what the fleet already shares — the number the
//! Closures summary table states and does not show.

use super::{
    color, gutter, legend, legend_bands, rect, svg_open, text, Band, D2Style, LEGEND_H, PAD, W,
};

const ROW_H: u64 = 26;
const BAR_H: u64 = 14;

/// One bar. An empty `bands` list draws no bar at all — the row is still
/// present, so a host with nothing to show says so rather than vanishing.
pub struct Row {
    pub label: String,
    pub bands: Vec<(Band, u64)>,
    /// Right-hand annotation: the formatted total, or why there is no bar.
    pub note: String,
}

/// A horizontal stacked bar chart, one row per entry, scaled to the largest
/// total. Returns a standalone SVG document carrying the AUTO marker.
pub fn bars(caption: &str, rows: &[Row], style: &D2Style) -> String {
    let ink = color(style, "chartInk", ("#333333", "#c9d1d9"));
    let muted = color(style, "chartMuted", ("#777777", "#8b949e"));
    let track = color(style, "chartTrack", ("#ebebeb", "#2a2a2e"));

    let label_w = gutter(rows.iter().map(|r| r.label.as_str()));
    let note_w = gutter(rows.iter().map(|r| r.note.as_str()));
    let plot_w = W.saturating_sub(label_w + note_w + PAD).max(1);

    // Only the bands that actually occur get a key: a two-host fleet cannot
    // produce a "shared by some" segment, and a lone measured host has
    // nothing to distinguish, so its single band needs no legend at all.
    let keys = legend_bands(
        rows.iter()
            .flat_map(|r| &r.bands)
            .filter(|(_, v)| *v > 0)
            .map(|(b, _)| *b),
    );

    let top = if keys.is_empty() { PAD } else { PAD + LEGEND_H };
    let h = top + ROW_H * rows.len() as u64 + PAD;
    let max = rows
        .iter()
        .map(|r| r.bands.iter().map(|(_, v)| v).sum::<u64>())
        .max()
        .unwrap_or(0);

    let mut o = svg_open(caption, h, style);

    legend(&mut o, &keys, label_w, style);

    for (i, row) in rows.iter().enumerate() {
        let cy = top + ROW_H * i as u64 + ROW_H / 2;
        let total: u64 = row.bands.iter().map(|(_, v)| v).sum();
        // Both gutters are right-aligned inward, so labels sit against their
        // bars and nothing rides the canvas edge, whatever the name lengths.
        text(
            &mut o,
            label_w - PAD,
            cy + 4,
            13,
            if total > 0 { ink } else { muted },
            true,
            &row.label,
        );
        if total > 0 && max > 0 {
            rect(&mut o, label_w, cy - BAR_H / 2, plot_w, BAR_H, track);
            // Scale the running total rather than each band, so the segments
            // tile exactly and the last one ends where the bar's own total
            // says it should, whatever the rounding did on the way.
            let (mut acc, mut x0) = (0u64, 0u64);
            for (band, value) in &row.bands {
                acc += value;
                let x1 = acc * plot_w / max;
                if x1 > x0 {
                    let (name, light, dark) = band.color();
                    let fill = color(style, name, (light, dark));
                    rect(&mut o, label_w + x0, cy - BAR_H / 2, x1 - x0, BAR_H, fill);
                }
                x0 = x1;
            }
        }
        text(
            &mut o,
            W - 4,
            cy + 4,
            12,
            if total > 0 { ink } else { muted },
            true,
            &row.note,
        );
    }

    o.push_str("</svg>");
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::out::MD_MARKER;

    fn row(label: &str, bands: Vec<(Band, u64)>, note: &str) -> Row {
        Row {
            label: label.into(),
            bands,
            note: note.into(),
        }
    }

    /// Widths of the bar rects only — legend swatches are square and BAR_H
    /// tall at most, so the height tells them apart.
    fn widths(svg: &str) -> Vec<u64> {
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
            .filter(|l| attr(l, "height") == Some(BAR_H))
            .filter_map(|l| attr(l, "width"))
            .collect()
    }

    #[test]
    fn segments_tile_the_bar_exactly() {
        // Three bands that do not divide evenly into the plot width: the
        // cumulative scaling must still make them add up to the track.
        let svg = bars(
            "t",
            &[row(
                "a",
                vec![
                    (Band::Shared, 100),
                    (Band::Partial, 100),
                    (Band::Unique, 100),
                ],
                "300 B",
            )],
            &D2Style::default(),
        );
        let w = widths(&svg);
        let (track, segments) = w.split_first().unwrap();
        assert_eq!(segments.iter().sum::<u64>(), *track, "{w:?}");
    }

    #[test]
    fn bars_are_scaled_to_the_largest_row() {
        let svg = bars(
            "t",
            &[
                row("big", vec![(Band::Solid, 100)], "100 B"),
                row("small", vec![(Band::Solid, 25)], "25 B"),
            ],
            &D2Style::default(),
        );
        // track, big, track, small
        let w = widths(&svg);
        assert_eq!(w[1], w[0], "the largest row fills the plot: {w:?}");
        assert_eq!(w[3], w[0] / 4, "{w:?}");
    }

    #[test]
    fn a_row_with_no_data_keeps_its_label_and_draws_nothing() {
        let svg = bars(
            "t",
            &[
                row("measured", vec![(Band::Solid, 10)], "10 B"),
                row("absent", vec![], "not measured"),
            ],
            &D2Style::default(),
        );
        assert!(svg.contains(">absent<"), "{svg}");
        assert!(svg.contains(">not measured<"), "{svg}");
        // Only the measured row contributes a track and a segment.
        assert_eq!(widths(&svg).len(), 2, "{svg}");
    }

    #[test]
    fn the_legend_lists_only_bands_that_occur() {
        let two_hosts = bars(
            "t",
            &[row(
                "a",
                vec![(Band::Shared, 10), (Band::Unique, 5)],
                "15 B",
            )],
            &D2Style::default(),
        );
        assert!(two_hosts.contains("shared by every host"));
        assert!(two_hosts.contains("unique to this host"));
        assert!(!two_hosts.contains("shared by some"), "{two_hosts}");

        // A single measured host has nothing to compare, so no key at all.
        let alone = bars(
            "t",
            &[row("a", vec![(Band::Solid, 10)], "10 B")],
            &D2Style::default(),
        );
        assert!(!alone.contains("closure<"), "{alone}");
    }

    #[test]
    fn labels_are_xml_escaped_and_the_marker_leads() {
        let svg = bars(
            "a & b",
            &[row("<host>", vec![(Band::Solid, 1)], "1 B")],
            &D2Style::default(),
        );
        assert!(svg.starts_with(MD_MARKER), "{svg}");
        assert!(svg.contains("<title>a &amp; b</title>"), "{svg}");
        assert!(svg.contains("&lt;host&gt;"), "{svg}");
    }

    #[test]
    fn a_color_override_reaches_a_chart_only_name() {
        let style = D2Style {
            colors: vec![("chartUnique".into(), "#ff0000".into())],
            ..D2Style::default()
        };
        let svg = bars("t", &[row("a", vec![(Band::Unique, 1)], "1 B")], &style);
        assert!(svg.contains("#ff0000"), "{svg}");
    }
}
