use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::ops::Range;

pub(super) fn unmask(svg: &str) -> String {
    let masks = masks(svg);
    let mut out = svg.to_string();
    for (at, m) in masks.iter().rev() {
        out.replace_range(at.clone(), &m.clip_path());
    }
    for (_, m) in &masks {
        out = out.replace(&format!(" mask=\"url(#{})\"", m.id), &m.reference());
    }
    out
}

fn masks(svg: &str) -> Vec<(Range<usize>, Mask)> {
    let mut reader = Reader::from_str(svg);
    let mut found = Vec::new();
    loop {
        let start = reader.buffer_position() as usize;
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == "mask" => {
                if let Some(m) = mask(&mut reader, &e) {
                    found.push((start..reader.buffer_position() as usize, m));
                }
            }
            Ok(Event::Eof) | Err(_) => return found,
            _ => {}
        }
    }
}

fn mask(reader: &mut Reader<&[u8]>, open: &BytesStart) -> Option<Mask> {
    let id = attr(open, "id")?.to_string();
    let (mut base, mut holes) = (None, Vec::new());
    loop {
        match reader.read_event().ok()? {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == "rect" => {
                let rect = Rect::from_tag(&e)?;
                match (attr(&e, "fill")?.as_ref(), base) {
                    ("white", None) => base = Some(rect),
                    ("black", Some(_)) => holes.push(rect),
                    _ => return None,
                }
            }
            Event::End(e) if e.name().as_ref() == "rect" => {}
            Event::End(e) if e.name().as_ref() == "mask" => {
                return Some(Mask {
                    id,
                    base: base?,
                    holes,
                })
            }
            Event::Text(t) if t.trim().is_empty() => {}
            _ => return None,
        }
    }
}

fn attr<'a>(e: &'a BytesStart, name: &str) -> Option<std::borrow::Cow<'a, str>> {
    Some(e.try_get_attribute(name).ok()??.value)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rect {
    fn span(x: f64, y: f64, x2: f64, y2: f64) -> Rect {
        Rect {
            x,
            y,
            w: x2 - x,
            h: y2 - y,
        }
    }

    fn x2(&self) -> f64 {
        self.x + self.w
    }

    fn y2(&self) -> f64 {
        self.y + self.h
    }

    fn minus(self, hole: Rect) -> Vec<Rect> {
        let (ix, iy) = (self.x.max(hole.x), self.y.max(hole.y));
        let (ix2, iy2) = (self.x2().min(hole.x2()), self.y2().min(hole.y2()));
        if ix >= ix2 || iy >= iy2 {
            return vec![self];
        }
        [
            Rect::span(self.x, self.y, self.x2(), iy),
            Rect::span(self.x, iy2, self.x2(), self.y2()),
            Rect::span(self.x, iy, ix, iy2),
            Rect::span(ix2, iy, self.x2(), iy2),
        ]
        .into_iter()
        .filter(|r| r.w > 0.0 && r.h > 0.0)
        .collect()
    }

    fn from_tag(tag: &BytesStart) -> Option<Rect> {
        let num = |name: &str| attr(tag, name)?.parse::<f64>().ok();
        Some(Rect {
            x: num("x")?,
            y: num("y")?,
            w: num("width")?,
            h: num("height")?,
        })
    }
}

struct Mask {
    id: String,
    base: Rect,
    holes: Vec<Rect>,
}

impl Mask {
    fn reference(&self) -> String {
        if self.holes.is_empty() {
            return String::new();
        }
        format!(" clip-path=\"url(#{})\"", self.id)
    }

    fn clip_path(&self) -> String {
        if self.holes.is_empty() {
            return String::new();
        }
        let mut region = vec![self.base];
        for hole in &self.holes {
            region = region.into_iter().flat_map(|r| r.minus(*hole)).collect();
        }
        let d: Vec<String> = region
            .iter()
            .map(|r| format!("M{} {}h{}v{}h{}z", r.x, r.y, r.w, r.h, -r.w))
            .collect();
        format!(
            "<clipPath id=\"{}\"><path d=\"{}\"/></clipPath>",
            self.id,
            d.join("")
        )
    }
}

#[cfg(test)]
mod tests;
