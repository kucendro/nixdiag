use anyhow::Result;
use itertools::Itertools;
use quick_xml::events::{BytesCData, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use regex::Regex;
use std::sync::LazyLock;

static FONT_FACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@font-face\s*\{[^}]*\}").unwrap());

pub fn no_fonts(svg: &str) -> String {
    FONT_FACE.replace_all(svg, "").into_owned()
}

fn flat(e: &BytesStart) -> BytesStart<'static> {
    BytesStart::new(e.name().as_ref().to_string()).with_attributes(e.attributes().flatten())
}

pub fn one_line(svg: &str) -> Result<String> {
    let mut reader = Reader::from_str(svg);
    let mut w = Writer::new(Vec::new());
    loop {
        match reader.read_event()? {
            Event::Eof => break,
            Event::Comment(_) | Event::DocType(_) => {}
            Event::Text(t) if t.contains('\n') && t.trim().is_empty() => {}
            Event::Text(t) => {
                w.write_event(Event::Text(BytesText::from_escaped(t.trim_matches('\n'))))?
            }
            Event::CData(c) => {
                let text = c.into_inner();
                let folded = text
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .join(" ");
                w.write_event(Event::CData(BytesCData::new(folded)))?
            }
            Event::Start(e) => w.write_event(Event::Start(flat(&e)))?,
            Event::Empty(e) => w.write_event(Event::Empty(flat(&e)))?,
            e => w.write_event(e)?,
        }
    }
    Ok(String::from_utf8(w.into_inner())?)
}

#[cfg(test)]
mod tests {
    use super::{no_fonts, one_line};

    #[test]
    fn font_faces_go_and_other_rules_stay() {
        let css = "@font-face {\n font-family: f;\n src: url(\"data:x\");\n}.a{fill:red}@font-face{src:y}";
        assert_eq!(no_fonts(css), ".a{fill:red}");
    }

    #[test]
    fn cdata_folds_to_one_line() {
        let svg = "<style><![CDATA[\n.a {\n\n  fill: red;\n}\n\n.b{}\n]]></style>";
        assert_eq!(
            one_line(svg).unwrap(),
            "<style><![CDATA[.a { fill: red; } .b{}]]></style>"
        );
    }

    #[test]
    fn layout_whitespace_goes_and_label_spaces_stay() {
        let svg = "<?xml version=\"1.0\"?>\n<!DOCTYPE svg>\n<!-- c -->\n<svg a=\"1\"\n b=\"2\">\n<text>\nR &amp; D\n</text>\n</svg>\n";
        assert_eq!(
            one_line(svg).unwrap(),
            "<?xml version=\"1.0\"?><svg a=\"1\" b=\"2\"><text>R &amp; D</text></svg>"
        );
    }
}
