use super::{paint, Key, Style, CH, INSET, LABEL_PX, LEGEND_H, NOTE_PX, PAD, SWATCH, W};
use quick_xml::escape::escape;

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
    svg: String,
    style: &'a Style,
}

impl<'a> Canvas<'a> {
    pub fn new(caption: &str, h: u64, style: &'a Style) -> Self {
        let svg = format!(
            "\
             <svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {W} {h}\" \
             width=\"{W}\" height=\"{h}\" role=\"img\" \
             font-family=\"ui-sans-serif, system-ui, sans-serif\">\n\
             \x20 <title>{}</title>\n",
            escape(caption)
        );
        let mut c = Canvas { svg, style };
        if let Some(bg) = &style.background {
            c.rect(0, 0, W, h, bg);
        }
        c
    }

    pub fn rect(&mut self, x: u64, y: u64, w: u64, h: u64, fill: &str) {
        self.svg.push_str(&format!(
            "  <rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" fill=\"{fill}\"/>\n"
        ));
    }

    pub fn text(&mut self, x: u64, y: u64, size: u64, fill: &str, end: bool, s: &str) {
        let anchor = if end { " text-anchor=\"end\"" } else { "" };
        self.svg.push_str(&format!(
            "  <text x=\"{x}\" y=\"{y}\" font-size=\"{size}\" fill=\"{fill}\"{anchor}>{}</text>\n",
            escape(s)
        ));
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

    pub fn finish(mut self) -> String {
        self.svg.push_str("</svg>");
        self.svg
    }
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
