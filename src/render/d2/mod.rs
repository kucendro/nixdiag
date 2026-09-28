mod doc;

pub use doc::{quote, Class, Doc};

use super::out::Out;
use super::style::{Style, Theme};
use super::svg::{no_fonts, one_line};
use crate::conf::files::{diagram, SRC, THEME};
use crate::conf::palette::diagram as p;
use crate::conf::tools::D2 as ARGS;
use crate::text::d2 as t;
use crate::text::messages::{self as m, Fail};
use anyhow::{bail, Result};
use itertools::Itertools;
use std::io::{ErrorKind, Write};
use std::process::{Command, Stdio};

const CLASSES: &str = include_str!("theme.d2");

pub trait Diagram {
    fn stem(&self) -> String;
    fn draw(&self, doc: &mut Doc);
}

pub struct D2<'a> {
    pub out: &'a Out,
    pub style: &'a Style,
    pub svg: bool,
}

impl D2<'_> {
    fn theme(&self, theme: Theme) -> String {
        let vars = p::ALL
            .iter()
            .map(|c| t::var(c.name, &quote(self.style.paint(theme, c))))
            .join("\n");
        let fill = self
            .style
            .background
            .as_deref()
            .map(|bg| t::fill(&quote(bg)));
        [t::vars(&vars)]
            .into_iter()
            .chain(fill)
            .chain([CLASSES.into()])
            .join("\n")
    }

    pub fn write_theme(&self) -> Result<()> {
        self.out.write(THEME, &self.theme(self.style.theme))
    }

    pub fn render(&self, d: &dyn Diagram) -> Result<()> {
        let stem = d.stem();
        let mut doc = Doc::default();
        d.draw(&mut doc);
        self.out
            .write(format!("{stem}.d2"), &format!("{}\n\n{doc}", t::IMPORT))?;
        if !self.svg {
            return Ok(());
        }
        for theme in Theme::ALL {
            let Some(svg) = self.run(&stem, theme, &format!("{}\n\n{doc}", self.theme(theme)))?
            else {
                return Ok(());
            };
            let svg = one_line(&svg)?;
            if theme == self.style.theme {
                self.out.write(format!("{stem}.svg"), &svg)?;
            }
            let themed = diagram::themed(&stem, theme.name());
            self.out.sub(SRC).write(themed, &no_fonts(&svg))?;
        }
        Ok(())
    }

    fn run(&self, stem: &str, theme: Theme, source: &str) -> Result<Option<String>> {
        let spawned = Command::new("d2")
            .args(ARGS)
            .args(theme.d2())
            .args(["-", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let mut child = match spawned {
            Err(e) if e.kind() == ErrorKind::NotFound => {
                println!("{}", m::no_renderer("d2"));
                return Ok(None);
            }
            Err(e) => bail!(Fail::Render("d2", stem.into(), e)),
            Ok(c) => c,
        };
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(source.as_bytes())?;
        }
        let o = child.wait_with_output()?;
        if !o.status.success() {
            let stderr = String::from_utf8_lossy(&o.stderr).into_owned();
            bail!(Fail::RenderOutput("d2", stem.into(), stderr))
        }
        Ok(Some(String::from_utf8(o.stdout)?))
    }
}
