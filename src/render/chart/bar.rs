use super::{legend_bands, paint, Band, Canvas, Frame, Style};

const ROW_H: u64 = 26;
const BAR_H: u64 = 14;

pub struct Row {
    pub label: String,
    pub bands: Vec<(Band, u64)>,
    pub note: String,
}

impl Row {
    fn total(&self) -> u64 {
        self.bands.iter().map(|(_, v)| v).sum()
    }
}

pub fn bars(caption: &str, rows: &[Row], style: &Style) -> String {
    let track = style.color(&paint::TRACK);
    let keys = legend_bands(
        rows.iter()
            .flat_map(|r| &r.bands)
            .filter(|(_, v)| *v > 0)
            .map(|(b, _)| *b),
    );
    let f = Frame::new(
        rows.iter().map(|r| (r.label.as_str(), r.note.as_str())),
        &keys,
        ROW_H,
    );
    let max = rows.iter().map(Row::total).max().unwrap_or(0);

    let mut c = Canvas::new(caption, f.height(rows.len()), style);
    c.legend(&keys, f.label_w);
    for (i, row) in rows.iter().enumerate() {
        let total = row.total();
        c.row(&f, i, (&row.label, &row.note), total > 0, |c, cy| {
            if total == 0 || max == 0 {
                return;
            }
            c.rect(f.label_w, cy - BAR_H / 2, f.plot_w, BAR_H, track);
            let (mut acc, mut x0) = (0u64, 0u64);
            for (band, value) in &row.bands {
                acc += value;
                let x1 = acc * f.plot_w / max;
                if x1 > x0 {
                    let fill = style.color(&band.color());
                    c.rect(f.label_w + x0, cy - BAR_H / 2, x1 - x0, BAR_H, fill);
                }
                x0 = x1;
            }
        });
    }
    c.finish()
}

#[cfg(test)]
mod tests {
    use super::super::canvas::attr;
    use super::*;

    fn row(label: &str, bands: Vec<(Band, u64)>, note: &str) -> Row {
        Row {
            label: label.into(),
            bands,
            note: note.into(),
        }
    }

    fn widths(svg: &str) -> Vec<u64> {
        svg.lines()
            .filter(|l| l.contains("<rect"))
            .filter(|l| attr(l, "height") == Some(BAR_H))
            .filter_map(|l| attr(l, "width"))
            .collect()
    }

    #[test]
    fn segments_tile_the_bar_exactly() {
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
            &Style::default(),
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
            &Style::default(),
        );
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
            &Style::default(),
        );
        assert!(svg.contains(">absent<"), "{svg}");
        assert!(svg.contains(">not measured<"), "{svg}");
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
            &Style::default(),
        );
        assert!(two_hosts.contains(Band::Shared.legend()));
        assert!(two_hosts.contains(Band::Unique.legend()));
        assert!(!two_hosts.contains(Band::Partial.legend()), "{two_hosts}");

        let alone = bars(
            "t",
            &[row("a", vec![(Band::Solid, 10)], "10 B")],
            &Style::default(),
        );
        assert!(!alone.contains(Band::Solid.legend()), "{alone}");
    }

    #[test]
    fn a_color_override_reaches_a_chart_only_name() {
        let style = Style {
            colors: vec![("chartUnique".into(), "#ff0000".into())],
            ..Style::default()
        };
        let svg = bars("t", &[row("a", vec![(Band::Unique, 1)], "1 B")], &style);
        assert!(svg.contains("#ff0000"), "{svg}");
    }
}
