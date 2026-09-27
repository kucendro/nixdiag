use super::style::Style;
use super::unmask::unmask;
use crate::conf::palette::DIAGRAM;
use crate::conf::tools::D2_LAYOUT;
use crate::render::out::Out;
use crate::text::d2::DIRECTION;
use crate::text::{fill, messages as m};
use anyhow::{bail, Result};
use std::fs;
use std::io::ErrorKind;
use std::process::Command;

pub struct D2<'a> {
    pub out: &'a Out,
    pub style: &'a Style,
    pub svg: bool,
}

impl D2<'_> {
    pub fn preamble(&self) -> Vec<String> {
        let mut o = Vec::new();
        if let Some(bg) = &self.style.background {
            o.push(format!("style.fill: \"{bg}\""));
        }
        o.push("vars: {".into());
        for c in DIAGRAM {
            o.push(format!("  {}: \"{}\"", c.name, self.style.color(c)));
        }
        o.push("}".into());
        o.push(DIRECTION.into());
        o
    }

    pub fn write(&self, stem: &str, lines: &[String]) -> Result<()> {
        let (d2, svg) = (format!("{stem}.d2"), format!("{stem}.svg"));
        self.out.write(&d2, &lines.join("\n"))?;
        if !self.svg {
            return Ok(());
        }
        let run = Command::new("d2")
            .args(D2_LAYOUT)
            .args(self.style.theme.d2())
            .arg(self.out.root.join(&d2))
            .arg(self.out.root.join(&svg))
            .output();
        match run {
            Err(e) if e.kind() == ErrorKind::NotFound => {
                println!("{}", m::NO_D2);
                Ok(())
            }
            Err(e) => bail!(fill(
                m::D2_FAILED,
                &[("stem", stem), ("error", &e.to_string())]
            )),
            Ok(o) if !o.status.success() => bail!(fill(
                m::D2_FAILED_OUTPUT,
                &[
                    ("stem", stem),
                    ("stderr", &String::from_utf8_lossy(&o.stderr))
                ]
            )),
            Ok(_) => self
                .out
                .put(&svg, |p| fs::write(p, unmask(&fs::read_to_string(p)?))),
        }
    }
}
