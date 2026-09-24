//! d2 emission and SVG rendering.

use crate::render::out::Out;
use anyhow::{bail, Result};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

pub const D2_HEADER: [&str; 2] = [
    "# Auto-generated from the Nix config by nixdiag. Do not edit.",
    "# Regenerate: nixdiag gen",
];

/// Appearance knobs. Every diagram color comes from the `vars` palette
/// block, so a theme swaps the whole set and any single name can be
/// overridden. One build renders one theme: d2 cannot switch explicitly
/// styled colors with the viewer's color scheme (d2lang/d2#831).
#[derive(Default)]
pub struct D2Style {
    pub dark: bool,
    pub background: Option<String>,
    /// palette overrides (name, color); later entries win
    pub colors: Vec<(String, String)>,
}

/// (name, light, dark)
pub const PALETTE: &[(&str, &str, &str)] = &[
    ("appFill", "#e6f0ff", "#1c2e4a"),
    ("appStroke", "#4a76c4", "#7fa7e8"),
    ("infraFill", "#ffe9cc", "#4a3413"),
    ("infraStroke", "#c47a29", "#d9995a"),
    ("baseFill", "#f0f0f0", "#2a2a2e"),
    ("baseStroke", "#999", "#666"),
    ("hostFill", "#fbfbfe", "#181825"),
    ("hostStroke", "#333", "#a6adc8"),
    ("progFill", "#eaffea", "#1e3a24"),
    ("hostCloud", "#fff3cd", "#3d3517"),
    ("public", "#c0392b", "#e74c3c"),
    ("lan", "#27893f", "#2ecc71"),
    ("mesh", "#4a76c4", "#7fa7e8"),
];

/// Resolve one color: an explicit `--color NAME=#hex` wins, then the built-in
/// light/dark pair for `name` in `PALETTE`, then `default`.
///
/// The `default` arm is what lets `render::chart` have its own tunable color
/// names without adding `PALETTE` entries. `vars_block` writes the whole
/// palette into every diagram, so a new entry would churn every snapshot in
/// this repo and in every consumer's committed docs; a name that resolves
/// only through `default` is still overridable but appears nowhere.
pub fn color<'a>(style: &'a D2Style, name: &str, default: (&'a str, &'a str)) -> &'a str {
    if let Some((_, v)) = style.colors.iter().rev().find(|(n, _)| n == name) {
        return v;
    }
    let (light, dark) = PALETTE
        .iter()
        .find(|(n, _, _)| *n == name)
        .map_or(default, |(_, l, d)| (*l, *d));
    if style.dark {
        dark
    } else {
        light
    }
}

pub fn vars_block(style: &D2Style) -> Vec<String> {
    let mut o = vec!["vars: {".to_string()];
    for (name, light, dark) in PALETTE {
        o.push(format!(
            "  {name}: \"{}\"",
            color(style, name, (light, dark))
        ));
    }
    o.push("}".into());
    o
}

pub fn write_and_render(
    out: &mut Out,
    stem: &str,
    lines: &[String],
    render_svg: bool,
    style: &D2Style,
) -> Result<()> {
    let mut lines = lines.to_vec();
    if let Some(bg) = &style.background {
        lines.insert(D2_HEADER.len(), format!("style.fill: \"{bg}\""));
    }
    let d2_rel = PathBuf::from(format!("{stem}.d2"));
    out.write_auto(&d2_rel, &lines.join("\n"))?;
    if !render_svg {
        return Ok(());
    }
    let svg_rel = PathBuf::from(format!("{stem}.svg"));
    let d2_path = out.root.join(&d2_rel);
    let svg_path = out.root.join(&svg_rel);
    let mut cmd = Command::new("d2");
    cmd.args(["--layout", "elk"]);
    if style.dark {
        cmd.args(["--theme", "200"]); // label/text colors for dark canvases
    }
    let run = cmd.arg(&d2_path).arg(&svg_path).output();
    match run {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            println!("(d2 binary not on PATH -- skipped SVG render)");
        }
        Err(e) => bail!("d2 render of {stem}.d2 failed: {e}"),
        Ok(o) if !o.status.success() => {
            bail!(
                "d2 render of {stem}.d2 failed:\n{}",
                String::from_utf8_lossy(&o.stderr)
            );
        }
        Ok(_) => {
            let svg = std::fs::read_to_string(&svg_path)?;
            std::fs::write(&svg_path, unmask(&svg))?;
            println!("wrote {}", svg_path.display());
            out.record_svg(&svg_rel);
        }
    }
    Ok(())
}

/// Rewrite d2's edge-label mask into a clip path.
///
/// d2 keeps each edge line from running through its own label with an SVG
/// `mask`: one white rect over the canvas, one black rect per label, and
/// `mask="url(#…)"` on every connection path — with or without labels.
/// A mask has no vector form in PDF, so Chrome's print backend rasterises
/// each masked path into a page-wide bitmap at 300 dpi, with an alpha
/// image beside it; a wiki printed to PDF grew by ~30 KB per edge and a
/// 24-page fleet came out at 6.4 MB, 5 MB of it 460 near-empty bitmaps.
/// The same region as a `clipPath` is a plain PDF clip, kept as vectors.
///
/// The region is emitted as one `<path>` of disjoint rectangles (the
/// canvas minus the union of the labels): a single child is what keeps
/// Chrome on its path-based clip rather than the mask fallback it uses
/// past a few dozen children, and disjoint pieces are exact under any fill
/// rule even where two labels overlap, which even-odd would flip back to
/// visible. A mask with no label rects, and the attributes pointing at it,
/// are dropped outright. Anything shaped differently from d2's pattern —
/// a non-rect child, an unexpected fill, a white rect after a black one —
/// is left untouched, so an upstream change degrades to today's output
/// rather than to a wrong picture.
pub fn unmask(svg: &str) -> String {
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
    fn x2(&self) -> f64 {
        self.x + self.w
    }

    fn y2(&self) -> f64 {
        self.y + self.h
    }

    /// `self` minus `hole`, as up to four disjoint rectangles.
    fn minus(self, hole: Rect) -> Vec<Rect> {
        let (ix, iy) = (self.x.max(hole.x), self.y.max(hole.y));
        let (ix2, iy2) = (self.x2().min(hole.x2()), self.y2().min(hole.y2()));
        if ix >= ix2 || iy >= iy2 {
            return vec![self];
        }
        let mut out = Vec::with_capacity(4);
        if iy > self.y {
            out.push(Rect {
                x: self.x,
                y: self.y,
                w: self.w,
                h: iy - self.y,
            });
        }
        if iy2 < self.y2() {
            out.push(Rect {
                x: self.x,
                y: iy2,
                w: self.w,
                h: self.y2() - iy2,
            });
        }
        if ix > self.x {
            out.push(Rect {
                x: self.x,
                y: iy,
                w: ix - self.x,
                h: iy2 - iy,
            });
        }
        if ix2 < self.x2() {
            out.push(Rect {
                x: ix2,
                y: iy,
                w: self.x2() - ix2,
                h: iy2 - iy,
            });
        }
        out
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

/// One d2 mask, understood: the canvas rect and the label rects cut out of it.
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

/// The value of attribute `name` in one tag's text. The leading space is
/// what keeps `width` from matching inside `stroke-width`.
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

/// Parse a `<mask …>…</mask>` element that follows d2's pattern exactly:
/// one white rect first, then only black rects, nothing else inside.
fn parse_mask(elem: &str) -> Option<Mask> {
    let open_end = elem.find('>')?;
    let id = attr(&elem[..open_end], "id")?.to_string();
    let body = &elem[open_end + 1..elem.len() - "</mask>".len()];
    let mut base = None;
    let mut holes = Vec::new();
    let mut rest = body;
    loop {
        let text_end = rest.find('<').unwrap_or(rest.len());
        if !rest[..text_end].trim().is_empty() {
            return None;
        }
        if text_end == rest.len() {
            break;
        }
        let tag_end = rest[text_end..].find('>')? + text_end;
        let tag = &rest[text_end..tag_end];
        rest = &rest[tag_end + 1..];
        if tag == "</rect" {
            continue;
        }
        if !tag.starts_with("<rect ") {
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
mod tests {
    use super::*;

    fn r(x: f64, y: f64, w: f64, h: f64) -> Rect {
        Rect { x, y, w, h }
    }

    fn area(rects: &[Rect]) -> f64 {
        rects.iter().map(|r| r.w * r.h).sum()
    }

    #[test]
    fn minus_leaves_a_disjoint_rect_alone() {
        assert_eq!(
            r(0.0, 0.0, 10.0, 10.0).minus(r(20.0, 20.0, 5.0, 5.0)),
            vec![r(0.0, 0.0, 10.0, 10.0)]
        );
    }

    #[test]
    fn minus_cuts_an_interior_hole_into_four_bands() {
        let pieces = r(0.0, 0.0, 10.0, 10.0).minus(r(2.0, 3.0, 4.0, 5.0));
        assert_eq!(pieces.len(), 4);
        assert_eq!(area(&pieces), 100.0 - 20.0);
        assert_eq!(pieces[0], r(0.0, 0.0, 10.0, 3.0));
        assert_eq!(pieces[1], r(0.0, 8.0, 10.0, 2.0));
        assert_eq!(pieces[2], r(0.0, 3.0, 2.0, 5.0));
        assert_eq!(pieces[3], r(6.0, 3.0, 4.0, 5.0));
    }

    #[test]
    fn minus_clips_a_hole_that_crosses_the_edge() {
        let pieces = r(0.0, 0.0, 10.0, 10.0).minus(r(8.0, -5.0, 10.0, 8.0));
        assert_eq!(area(&pieces), 100.0 - 2.0 * 3.0);
        assert!(pieces.iter().all(|p| p.w > 0.0 && p.h > 0.0));
    }

    #[test]
    fn overlapping_holes_are_subtracted_once() {
        let m = Mask {
            id: "m".into(),
            base: r(0.0, 0.0, 10.0, 10.0),
            holes: vec![r(1.0, 1.0, 4.0, 4.0), r(3.0, 3.0, 4.0, 4.0)],
        };
        let mut region = vec![m.base];
        for hole in &m.holes {
            region = region.into_iter().flat_map(|p| p.minus(*hole)).collect();
        }
        // union of the two 4x4 holes overlapping on a 2x2 square
        assert_eq!(area(&region), 100.0 - (16.0 + 16.0 - 4.0));
    }

    const D2: &str = concat!(
        "<svg>\n",
        "<mask id=\"d2-1\" maskUnits=\"userSpaceOnUse\" x=\"-89\" y=\"-89\" width=\"200\" height=\"200\">\n",
        "<rect x=\"-89\" y=\"-89\" width=\"200\" height=\"200\" fill=\"white\"></rect>\n",
        "<rect x=\"10.000000\" y=\"20.000000\" width=\"30\" height=\"21\" fill=\"black\"></rect>\n",
        "</mask>\n",
        "<path d=\"M 0 0 L 1 1\" class=\"connection\" marker-end=\"url(#mk-d2-1-9)\" mask=\"url(#d2-1)\" />\n",
        "<path d=\"M 0 0 L 2 2\" class=\"connection\" mask=\"url(#d2-1)\" />\n",
        "</svg>\n"
    );

    #[test]
    fn label_mask_becomes_one_clip_path_of_disjoint_rects() {
        let out = unmask(D2);
        assert!(!out.contains("<mask"), "{out}");
        assert!(!out.contains(" mask="), "{out}");
        assert_eq!(out.matches(" clip-path=\"url(#d2-1)\"").count(), 2);
        assert_eq!(out.matches("<clipPath id=\"d2-1\">").count(), 1);
        assert_eq!(
            out.matches("<path d=\"M-89 -89h200v109h-200z").count(),
            1,
            "{out}"
        );
        assert!(out.contains("marker-end=\"url(#mk-d2-1-9)\""));
    }

    #[test]
    fn mask_without_labels_is_dropped_with_its_references() {
        let svg = D2.replace(
            "<rect x=\"10.000000\" y=\"20.000000\" width=\"30\" height=\"21\" fill=\"black\"></rect>\n",
            "",
        );
        let out = unmask(&svg);
        assert!(!out.contains("mask"), "{out}");
        assert!(!out.contains("clip"), "{out}");
        assert_eq!(out.matches("<path d=\"M 0 0").count(), 2);
    }

    #[test]
    fn unfamiliar_mask_is_left_alone() {
        for foreign in [
            D2.replace("fill=\"black\"", "fill=\"#000\""),
            D2.replace(
                "<rect x=\"10.000000\"",
                "<circle r=\"3\"/><rect x=\"10.000000\"",
            ),
            D2.replace("fill=\"white\"", "fill=\"black\""),
        ] {
            assert_eq!(unmask(&foreign), foreign);
        }
    }

    #[test]
    fn attr_needs_the_leading_space() {
        let tag = "<path style=\"stroke-width:2\" width=\"7\"";
        assert_eq!(attr(tag, "width"), Some("7"));
        assert_eq!(attr("<path stroke-width=\"2\"", "width"), None);
    }
}
