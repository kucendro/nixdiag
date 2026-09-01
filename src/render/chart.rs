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

use super::d2::{color, D2Style};
use super::out::MD_MARKER;
use crate::util::human_size;

/// Canvas width. mdBook's content column is a little under 750px, so a
/// fixed-width chart lands unscaled at the default theme and shrinks
/// proportionally on anything narrower.
const W: u64 = 720;
const ROW_H: u64 = 26;
const BAR_H: u64 = 14;
const PAD: u64 = 8;
/// Approximate advance width of the label font at 13px. Only used to size
/// gutters, so being a little generous costs nothing but whitespace.
const CH: u64 = 7;
const LEGEND_H: u64 = 24;
const SWATCH: u64 = 10;

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

/// One bar. An empty `bands` list draws no bar at all — the row is still
/// present, so a host with nothing to show says so rather than vanishing.
pub struct Row {
    pub label: String,
    pub bands: Vec<(Band, u64)>,
    /// Right-hand annotation: the formatted total, or why there is no bar.
    pub note: String,
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
    let mut legend: Vec<Band> = Vec::new();
    for (band, value) in rows.iter().flat_map(|r| &r.bands) {
        if *value > 0 && !legend.contains(band) {
            legend.push(*band);
        }
    }
    if legend == [Band::Solid] {
        legend.clear();
    }

    let top = if legend.is_empty() {
        PAD
    } else {
        PAD + LEGEND_H
    };
    let h = top + ROW_H * rows.len() as u64 + PAD;
    let max = rows
        .iter()
        .map(|r| r.bands.iter().map(|(_, v)| v).sum::<u64>())
        .max()
        .unwrap_or(0);

    let mut o = svg_open(caption, h, style);

    let mut x = label_w;
    for band in &legend {
        let (name, light, dark) = band.color();
        rect(
            &mut o,
            x,
            PAD + 2,
            SWATCH,
            SWATCH,
            color(style, name, (light, dark)),
        );
        text(
            &mut o,
            x + SWATCH + 5,
            PAD + 11,
            12,
            muted,
            false,
            band.legend(),
        );
        x += SWATCH + 5 + CH * band.legend().len() as u64 + 14;
    }

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

/// Canvas height for a treemap. Wider than tall, so the first row the
/// algorithm lays down runs along the vertical side and the largest tile
/// lands top-left, where a reader starts.
const TREE_H: u64 = 400;

/// The longest prefix of `s` that fits `cap` characters, elided when it had
/// to cut. Below six characters a package name stops being recognisable, so
/// nothing is drawn rather than a stub.
fn fit_label(s: &str, cap: usize) -> Option<String> {
    let n = s.chars().count();
    if cap >= n {
        return Some(s.to_string());
    }
    if cap < 6 {
        return None;
    }
    Some(s.chars().take(cap - 1).collect::<String>() + "\u{2026}")
}

/// One rectangle of a treemap.
pub struct Tile {
    pub label: String,
    pub value: u64,
    pub band: Band,
}

/// The worst aspect ratio in a row of `areas` laid along a side of length
/// `side` (Bruls, Huizing and van Wijk). Lower is squarer.
fn worst(side: f64, sum: f64, lo: f64, hi: f64) -> f64 {
    if sum <= 0.0 || lo <= 0.0 || side <= 0.0 {
        return f64::INFINITY;
    }
    let (s2, w2) = (sum * sum, side * side);
    (w2 * hi / s2).max(s2 / (w2 * lo))
}

/// Squarified treemap layout: repeatedly grow a row along the shorter side of
/// what is left, for as long as adding the next area improves the worst
/// aspect ratio in that row, then place the row and recurse on the remainder.
///
/// `areas` must be descending — that is what makes the result squarish.
///
/// The layout runs in f64 because aspect ratios are divisions and integer
/// cross-multiplication buys nothing when every coordinate is rounded to a
/// whole pixel before it reaches the output. It stays deterministic: the same
/// values drive the same IEEE-754 operations in the same order.
fn squarify(areas: &[f64], rect: (f64, f64, f64, f64)) -> Vec<(f64, f64, f64, f64)> {
    let (mut x, mut y, mut w, mut h) = rect;
    let mut out = Vec::with_capacity(areas.len());
    let mut i = 0;
    while i < areas.len() {
        if w <= 0.0 || h <= 0.0 {
            out.extend(std::iter::repeat_n((x, y, 0.0, 0.0), areas.len() - i));
            break;
        }
        let side = w.min(h);
        let (mut j, mut sum, mut lo, mut hi) = (i, 0.0f64, f64::INFINITY, 0.0f64);
        let mut best = f64::INFINITY;
        while j < areas.len() {
            let (s, l, g) = (sum + areas[j], lo.min(areas[j]), hi.max(areas[j]));
            let cand = worst(side, s, l, g);
            // The first area always joins: a row of one is the baseline the
            // rest are measured against.
            if j > i && cand > best {
                break;
            }
            best = cand;
            sum = s;
            lo = l;
            hi = g;
            j += 1;
        }
        if w <= h {
            let rh = sum / w;
            let mut cx = x;
            for a in &areas[i..j] {
                let tw = if sum > 0.0 { a / sum * w } else { 0.0 };
                out.push((cx, y, tw, rh));
                cx += tw;
            }
            y += rh;
            h -= rh;
        } else {
            let rw = sum / h;
            let mut cy = y;
            for a in &areas[i..j] {
                let th = if sum > 0.0 { a / sum * h } else { 0.0 };
                out.push((x, cy, rw, th));
                cy += th;
            }
            x += rw;
            w -= rw;
        }
        i = j;
    }
    out
}

/// A treemap: rectangle area proportional to `value`, coloured by band.
///
/// Sorts internally, so the caller's order does not matter.
pub fn treemap(caption: &str, tiles: &[Tile], style: &D2Style) -> String {
    let muted = color(style, "chartMuted", ("#777777", "#8b949e"));
    // Tile text sits on a band colour, so it contrasts with the fill rather
    // than following the theme's ink: the light palette's fills are dark and
    // the dark palette's are light.
    let tile_ink = color(style, "chartTileInk", ("#ffffff", "#14181f"));

    let mut order: Vec<&Tile> = tiles.iter().filter(|t| t.value > 0).collect();
    order.sort_by(|a, b| b.value.cmp(&a.value).then(a.label.cmp(&b.label)));

    let mut legend: Vec<Band> = Vec::new();
    for t in &order {
        if !legend.contains(&t.band) {
            legend.push(t.band);
        }
    }
    if legend == [Band::Solid] {
        legend.clear();
    }
    let top = if legend.is_empty() {
        PAD
    } else {
        PAD + LEGEND_H
    };
    let h = top + TREE_H + PAD;

    let mut o = svg_open(caption, h, style);
    let mut x = 0;
    for band in &legend {
        let (name, light, dark) = band.color();
        rect(
            &mut o,
            x,
            PAD + 2,
            SWATCH,
            SWATCH,
            color(style, name, (light, dark)),
        );
        text(
            &mut o,
            x + SWATCH + 5,
            PAD + 11,
            12,
            muted,
            false,
            band.legend(),
        );
        x += SWATCH + 5 + CH * band.legend().len() as u64 + 14;
    }

    let total: f64 = order.iter().map(|t| t.value as f64).sum();
    let (pw, ph) = (W as f64, TREE_H as f64);
    let areas: Vec<f64> = order
        .iter()
        .map(|t| t.value as f64 / total * pw * ph)
        .collect();

    for (t, (rx, ry, rw, rh)) in order
        .iter()
        .zip(squarify(&areas, (0.0, top as f64, pw, ph)))
    {
        // A one-pixel inset is the whole separator: gaps show the page
        // through, so the tiles need no strokes.
        let (px, py) = (rx.round() as u64 + 1, ry.round() as u64 + 1);
        let (tw, th) = (
            (rw.round() as u64).saturating_sub(2),
            (rh.round() as u64).saturating_sub(2),
        );
        if tw == 0 || th == 0 {
            continue;
        }
        let (name, light, dark) = t.band.color();
        rect(&mut o, px, py, tw, th, color(style, name, (light, dark)));
        // Elide rather than clip, and only when there is a line's height to
        // put it on. Losing the label on a large tile is the worse failure:
        // the biggest rectangle is the one a reader most wants named.
        let cap = (tw.saturating_sub(8) / CH) as usize;
        if th >= 18 {
            if let Some(label) = fit_label(&t.label, cap) {
                text(&mut o, px + 4, py + 15, 13, tile_ink, false, &label);
            }
            let size = human_size(t.value);
            if th >= 34 && cap >= size.chars().count() {
                text(&mut o, px + 4, py + 30, 12, tile_ink, false, &size);
            }
        }
    }

    o.push_str("</svg>");
    o
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn tile(label: &str, value: u64, band: Band) -> Tile {
        Tile {
            label: label.into(),
            value,
            band,
        }
    }

    /// Every `<rect>` as (x, y, w, h).
    fn rects(svg: &str) -> Vec<(u64, u64, u64, u64)> {
        let attr = |l: &str, k: &str| -> u64 {
            l.split(&format!("{k}=\""))
                .nth(1)
                .and_then(|r| r.split('"').next())
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
        };
        svg.lines()
            .filter(|l| l.contains("<rect"))
            .map(|l| {
                (
                    attr(l, "x"),
                    attr(l, "y"),
                    attr(l, "width"),
                    attr(l, "height"),
                )
            })
            .collect()
    }

    #[test]
    fn treemap_area_tracks_value_and_stays_on_canvas() {
        let svg = treemap(
            "t",
            &[
                tile("a", 600, Band::Solid),
                tile("b", 300, Band::Solid),
                tile("c", 100, Band::Solid),
            ],
            &D2Style::default(),
        );
        let r = rects(&svg);
        assert_eq!(r.len(), 3, "one rect per tile, no legend for Solid alone");
        let area: Vec<u64> = r.iter().map(|(_, _, w, h)| w * h).collect();
        // Ratios survive the rounding: b is about half of a, c about a sixth.
        assert!(
            (area[0] as f64 / area[1] as f64 - 2.0).abs() < 0.1,
            "{area:?}"
        );
        assert!(
            (area[0] as f64 / area[2] as f64 - 6.0).abs() < 0.3,
            "{area:?}"
        );
        // Nothing escapes the canvas.
        for (x, y, w, h) in &r {
            assert!(x + w <= W && y + h <= TREE_H + 2 * PAD, "{r:?}");
        }
    }

    #[test]
    fn the_largest_tile_lands_top_left() {
        let svg = treemap(
            "t",
            &[tile("small", 1, Band::Solid), tile("big", 99, Band::Solid)],
            &D2Style::default(),
        );
        // Sorted internally, so the caller's order does not matter.
        let first = svg.find(">big<").unwrap();
        assert!(first < svg.find(">small<").unwrap_or(usize::MAX), "{svg}");
        let (x, y, _, _) = rects(&svg)[0];
        assert_eq!((x, y), (1, PAD + 1), "inset by the one-pixel gap");
    }

    #[test]
    fn zero_valued_tiles_are_dropped_not_drawn_flat() {
        let svg = treemap(
            "t",
            &[tile("real", 10, Band::Solid), tile("empty", 0, Band::Solid)],
            &D2Style::default(),
        );
        assert_eq!(rects(&svg).len(), 1, "{svg}");
        assert!(!svg.contains(">empty<"), "{svg}");
    }

    #[test]
    fn treemap_bands_get_a_legend_like_the_bars() {
        let svg = treemap(
            "t",
            &[
                tile("mine", 10, Band::Unique),
                tile("ours", 10, Band::Shared),
                tile("3 more", 5, Band::Rest),
            ],
            &D2Style::default(),
        );
        for key in [
            "unique to this host",
            "shared by every host",
            "everything smaller",
        ] {
            assert!(svg.contains(key), "missing {key} in {svg}");
        }
    }

    #[test]
    fn labels_elide_rather_than_clip() {
        assert_eq!(fit_label("glibc", 10).as_deref(), Some("glibc"));
        assert_eq!(fit_label("glibc", 5).as_deref(), Some("glibc"));
        assert_eq!(
            fit_label("playwright-chromium-headless-shell", 12).as_deref(),
            Some("playwright-\u{2026}")
        );
        // Too narrow to recognise anything: draw nothing.
        assert_eq!(fit_label("playwright-chromium", 5), None);
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
