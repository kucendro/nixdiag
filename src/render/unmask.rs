pub(super) fn unmask(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    let mut rewrites: Vec<(String, bool)> = Vec::new();
    while let Some(i) = rest.find("<mask") {
        let after = rest[i + "<mask".len()..].chars().next();
        let Some(len) = rest[i..].find("</mask>") else {
            break;
        };
        let elem = &rest[i..i + len + "</mask>".len()];
        out.push_str(&rest[..i]);
        match parse_mask(elem).filter(|_| matches!(after, Some(' ') | Some('>'))) {
            Some(m) => {
                let keep = !m.holes.is_empty();
                if keep {
                    out.push_str(&m.clip_path());
                }
                rewrites.push((m.id, keep));
            }
            None => out.push_str(elem),
        }
        rest = &rest[i + elem.len()..];
    }
    out.push_str(rest);
    for (id, keep) in rewrites {
        let from = format!(" mask=\"url(#{id})\"");
        let to = if keep {
            format!(" clip-path=\"url(#{id})\"")
        } else {
            String::new()
        };
        out = out.replace(&from, &to);
    }
    out
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

    fn from_tag(tag: &str) -> Option<Rect> {
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
    fn clip_path(&self) -> String {
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

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

fn parse_mask(elem: &str) -> Option<Mask> {
    let open_end = elem.find('>')?;
    let id = attr(&elem[..open_end], "id")?.to_string();
    let body = &elem[open_end + 1..elem.len() - "</mask>".len()];
    let mut parts = body.split('<');
    if !parts.next()?.trim().is_empty() {
        return None;
    }
    let mut base = None;
    let mut holes = Vec::new();
    for part in parts {
        let (tag, text) = part.split_once('>')?;
        if !text.trim().is_empty() {
            return None;
        }
        if tag == "/rect" {
            continue;
        }
        if !tag.starts_with("rect ") {
            return None;
        }
        let rect = Rect::from_tag(tag)?;
        match (attr(tag, "fill")?, base) {
            ("white", None) => base = Some(rect),
            ("black", Some(_)) => holes.push(rect),
            _ => return None,
        }
    }
    Some(Mask {
        id,
        base: base?,
        holes,
    })
}

#[cfg(test)]
mod tests;
