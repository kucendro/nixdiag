use super::style::Style;
use crate::conf::palette::{diagram as p, Color};
use crate::conf::tools::{DOT, DOT_FONT};
use crate::render::out::Out;
use crate::text::messages::{self as m, Fail};
use anyhow::{bail, Result};
use dot_writer::{Attributes, DotWriter, RankDirection, Scope};
use quick_xml::escape::escape;
use std::io::ErrorKind;
use std::process::Command;

fn quote(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn id(s: &str) -> String {
    format!("\"{}\"", quote(s))
}

pub fn card(title: &str, fill: &str, stroke: &str, rows: &[(&str, &str)]) -> String {
    let rows: String = rows
        .iter()
        .map(|(bg, s)| format!(r#"<TR><TD BGCOLOR="{bg}">{}</TD></TR>"#, escape(*s)))
        .collect();
    format!(
        r#"<<TABLE STYLE="rounded" BORDER="1" COLOR="{stroke}" BGCOLOR="{fill}" CELLBORDER="0" CELLSPACING="3" CELLPADDING="4"><TR><TD>{}</TD></TR>{rows}</TABLE>>"#,
        escape(title)
    )
}

pub trait Paint: Attributes {
    fn fill(&mut self, c: &str) -> &mut Self {
        self.set("fillcolor", c, true)
    }

    fn stroke(&mut self, c: &str) -> &mut Self {
        self.set("color", c, true)
    }

    fn shape(&mut self, s: &str) -> &mut Self {
        self.set("shape", s, false)
    }

    fn text(&mut self, lines: &[&str]) -> &mut Self {
        let lines: Vec<String> = lines.iter().map(|l| quote(l)).collect();
        self.set("label", &lines.join("\\n"), true)
    }

    fn bold(&mut self, s: &str) -> &mut Self {
        self.set("label", &format!("<<B>{}</B>>", escape(s)), false)
    }
}

impl<T: Attributes> Paint for T {}

pub struct Dot<'a> {
    pub out: &'a Out,
    pub style: &'a Style,
    pub svg: bool,
}

impl Dot<'_> {
    pub fn color(&self, c: &Color) -> &str {
        self.style.color(c)
    }

    pub fn render(&self, stem: &str, draw: impl FnOnce(&mut Scope)) -> Result<()> {
        let mut bytes = Vec::new();
        {
            let mut w = DotWriter::from(&mut bytes);
            let mut g = w.digraph();
            let ink = self.color(&p::INK);
            let mut graph = g.graph_attributes();
            graph.set_rank_direction(RankDirection::LeftRight);
            graph.set_font(DOT_FONT).set("fontcolor", ink, true);
            if let Some(bg) = &self.style.background {
                graph.set("bgcolor", bg, true);
            }
            drop(graph);
            g.node_attributes()
                .set_font(DOT_FONT)
                .set("fontcolor", ink, true)
                .shape("box")
                .set("style", "rounded,filled", true)
                .fill(self.color(&p::BASE_FILL))
                .stroke(self.color(&p::BASE_STROKE));
            g.edge_attributes()
                .set_font(DOT_FONT)
                .set_font_size(11.0)
                .set("fontcolor", ink, true)
                .stroke(self.color(&p::LINE));
            draw(&mut g);
        }
        let dot = format!("{stem}.dot");
        self.out.write(&dot, &String::from_utf8(bytes)?)?;
        if !self.svg {
            return Ok(());
        }
        match Command::new("dot")
            .args(DOT)
            .arg(self.out.root.join(&dot))
            .output()
        {
            Err(e) if e.kind() == ErrorKind::NotFound => {
                println!("{}", m::no_renderer("dot"));
                Ok(())
            }
            Err(e) => bail!(Fail::Render("dot", stem.into(), e)),
            Ok(o) if !o.status.success() => {
                let stderr = String::from_utf8_lossy(&o.stderr).into_owned();
                bail!(Fail::RenderOutput("dot", stem.into(), stderr))
            }
            Ok(o) => self
                .out
                .write(format!("{stem}.svg"), &String::from_utf8(o.stdout)?),
        }
    }
}
