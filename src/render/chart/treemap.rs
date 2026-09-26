use super::{
    color, legend, legend_bands, rect, svg_open, text, Band, D2Style, CH, LEGEND_H, PAD, W,
};
use crate::util::human_size;

const TREE_H: u64 = 400;

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

pub struct Tile {
    pub label: String,
    pub value: u64,
    pub band: Band,
}

fn worst(side: f64, sum: f64, lo: f64, hi: f64) -> f64 {
    if sum <= 0.0 || lo <= 0.0 || side <= 0.0 {
        return f64::INFINITY;
    }
    let (s2, w2) = (sum * sum, side * side);
    (w2 * hi / s2).max(s2 / (w2 * lo))
}

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

pub fn treemap(caption: &str, tiles: &[Tile], style: &D2Style) -> String {
    let tile_ink = color(style, "chartTileInk", ("#ffffff", "#14181f"));

    let mut order: Vec<&Tile> = tiles.iter().filter(|t| t.value > 0).collect();
    order.sort_by(|a, b| b.value.cmp(&a.value).then(a.label.cmp(&b.label)));

    let keys = legend_bands(order.iter().map(|t| t.band));
    let top = if keys.is_empty() { PAD } else { PAD + LEGEND_H };
    let h = top + TREE_H + PAD;

    let mut o = svg_open(caption, h, style);
    legend(&mut o, &keys, 0, style);

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

    fn tile(label: &str, value: u64, band: Band) -> Tile {
        Tile {
            label: label.into(),
            value,
            band,
        }
    }

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
        assert!(
            (area[0] as f64 / area[1] as f64 - 2.0).abs() < 0.1,
            "{area:?}"
        );
        assert!(
            (area[0] as f64 / area[2] as f64 - 6.0).abs() < 0.3,
            "{area:?}"
        );
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
        assert_eq!(fit_label("playwright-chromium", 5), None);
    }
}
