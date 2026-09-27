use super::{legend_bands, paint, top, Band, Canvas, Style, CH, INSET, LABEL_PX, NOTE_PX, PAD, W};
use crate::human::Bytes;
use streemap::{squarify, Rect};

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

pub fn treemap(caption: &str, tiles: &[Tile], style: &Style) -> String {
    let tile_ink = style.color(&paint::TILE_INK);

    let mut order: Vec<&Tile> = tiles.iter().filter(|t| t.value > 0).collect();
    order.sort_by(|a, b| b.value.cmp(&a.value).then(a.label.cmp(&b.label)));

    let keys = legend_bands(order.iter().map(|t| t.band));
    let top = top(&keys);

    let mut c = Canvas::new(caption, top + TREE_H + PAD, style);
    c.legend(&keys, 0);

    let zero = Rect {
        x: 0.0,
        y: 0.0,
        w: 0.0,
        h: 0.0,
    };
    let mut cells: Vec<(&Tile, Rect<f64>)> = order.iter().map(|t| (*t, zero)).collect();
    let (y, w, h) = (top as f64, W as f64, TREE_H as f64);
    squarify(
        Rect { x: 0.0, y, w, h },
        &mut cells,
        |(t, _)| t.value as f64,
        |(_, r), n| *r = n,
    );

    for (t, r) in &cells {
        let (px, py) = (r.x.round() as u64 + 1, r.y.round() as u64 + 1);
        let (tw, th) = (
            (r.w.round() as u64).saturating_sub(2),
            (r.h.round() as u64).saturating_sub(2),
        );
        if tw == 0 || th == 0 {
            continue;
        }
        c.rect(px, py, tw, th, style.color(&t.band.color()));
        let cap = (tw.saturating_sub(2 * INSET) / CH) as usize;
        if th >= 18 {
            if let Some(label) = fit_label(&t.label, cap) {
                c.text(px + INSET, py + 15, LABEL_PX, tile_ink, false, &label);
            }
            let size = Bytes(t.value).to_string();
            if th >= 34 && cap >= size.chars().count() {
                c.text(px + INSET, py + 30, NOTE_PX, tile_ink, false, &size);
            }
        }
    }
    c.finish()
}

#[cfg(test)]
mod tests {
    use super::super::canvas::{attr, texts};
    use super::*;

    fn tile(label: &str, value: u64, band: Band) -> Tile {
        Tile {
            label: label.into(),
            value,
            band,
        }
    }

    fn rects(svg: &str) -> Vec<(u64, u64, u64, u64)> {
        let at = |l: &str, k: &str| attr(l, k).unwrap_or(0);
        svg.lines()
            .filter(|l| l.contains("<rect"))
            .map(|l| (at(l, "x"), at(l, "y"), at(l, "width"), at(l, "height")))
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
            &Style::default(),
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
            &Style::default(),
        );
        assert_eq!(texts(&svg).first(), Some(&"big"), "{svg}");
        let (x, y, _, _) = rects(&svg)[0];
        assert_eq!((x, y), (1, PAD + 1), "inset by the one-pixel gap");
    }

    #[test]
    fn zero_valued_tiles_are_dropped_not_drawn_flat() {
        let svg = treemap(
            "t",
            &[tile("real", 10, Band::Solid), tile("empty", 0, Band::Solid)],
            &Style::default(),
        );
        assert_eq!(rects(&svg).len(), 1, "{svg}");
        assert!(!texts(&svg).contains(&"empty"), "{svg}");
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
            &Style::default(),
        );
        for key in [Band::Unique, Band::Shared, Band::Rest] {
            assert!(svg.contains(key.legend()), "missing {key:?} in {svg}");
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
