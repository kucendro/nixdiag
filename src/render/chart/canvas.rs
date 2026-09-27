use super::{paint, Key, Style, CH, FONT, INSET, LABEL_PX, LEGEND_H, NOTE_PX, PAD, SWATCH, W};
use svg::node::element::{Rectangle, Text, Title};
use svg::{Document, Node};

fn gutter<'a>(strings: impl Iterator<Item = &'a str>) -> u64 {
    CH * strings.map(|s| s.chars().count()).max().unwrap_or(0) as u64 + 12
}

pub fn top(keys: &[Key]) -> u64 {
    PAD + if keys.is_empty() { 0 } else { LEGEND_H }
}

pub struct Frame {
    pub label_w: u64,
    pub plot_w: u64,
    top: u64,
    row_h: u64,
}

impl Frame {
    pub fn new<'a>(
        rows: impl Iterator<Item = (&'a str, &'a str)> + Clone,
        keys: &[Key],
        row_h: u64,
    ) -> Frame {
        let label_w = gutter(rows.clone().map(|(label, _)| label));
        let note_w = gutter(rows.map(|(_, note)| note));
        Frame {
            label_w,
            plot_w: W.saturating_sub(label_w + note_w + PAD).max(1),
            top: top(keys),
            row_h,
        }
    }

    pub fn height(&self, rows: usize) -> u64 {
        self.top + self.row_h * rows as u64 + PAD
    }
}

pub struct Canvas<'a> {
    doc: Document,
    style: &'a Style,
}

impl<'a> Canvas<'a> {
    pub fn new(caption: &str, h: u64, style: &'a Style) -> Self {
        let doc = Document::new()
            .set("viewBox", (0, 0, W, h))
            .set("width", W)
            .set("height", h)
            .set("role", "img")
            .set("font-family", FONT)
            .add(Title::new(caption));
        let mut c = Canvas { doc, style };
        if let Some(bg) = &style.background {
            c.rect(0, 0, W, h, bg);
        }
        c
    }

    pub fn rect(&mut self, x: u64, y: u64, w: u64, h: u64, fill: &str) {
        let r = Rectangle::new().set("x", x).set("y", y);
        self.doc
            .append(r.set("width", w).set("height", h).set("fill", fill));
    }

    pub fn text(&mut self, x: u64, y: u64, size: u64, fill: &str, end: bool, s: &str) {
        let t = Text::new(s).set("x", x).set("y", y).set("font-size", size);
        let t = t.set("fill", fill);
        self.doc
            .append(if end { t.set("text-anchor", "end") } else { t });
    }

    pub fn legend(&mut self, keys: &[Key], mut x: u64) {
        let muted = self.style.color(&paint::MUTED);
        for key in keys {
            let swatch = self.style.color(&key.color);
            self.rect(x, PAD + 2, SWATCH, SWATCH, swatch);
            self.text(x + SWATCH + 5, PAD + 11, NOTE_PX, muted, false, key.label);
            x += SWATCH + 5 + CH * key.label.len() as u64 + 14;
        }
    }

    pub fn row(
        &mut self,
        f: &Frame,
        i: usize,
        (label, note): (&str, &str),
        active: bool,
        plot: impl FnOnce(&mut Self, u64),
    ) {
        let ink = if active { paint::INK } else { paint::MUTED };
        let ink = self.style.color(&ink);
        let cy = f.top + f.row_h * i as u64 + f.row_h / 2;
        self.text(f.label_w - PAD, cy + INSET, LABEL_PX, ink, true, label);
        plot(self, cy);
        self.text(W - INSET, cy + INSET, NOTE_PX, ink, true, note);
    }

    pub fn finish(self) -> String {
        self.doc.to_string()
    }
}

#[cfg(test)]
pub fn texts(svg: &str) -> Vec<&str> {
    let lines: Vec<&str> = svg.lines().collect();
    lines
        .windows(2)
        .filter(|w| w[0].starts_with("<text"))
        .map(|w| w[1])
        .collect()
}

#[cfg(test)]
pub fn attr(line: &str, key: &str) -> Option<u64> {
    line.split(&format!("{key}=\""))
        .nth(1)?
        .split('"')
        .next()?
        .parse()
        .ok()
}
