use super::style::Style;
use super::unmask::unmask;
use crate::conf::palette::DIAGRAM;
use crate::conf::tools::D2_LAYOUT;
use crate::render::out::Out;
use crate::text::d2::DIRECTION;
use crate::text::{fill, messages as m};
use anyhow::{bail, Result};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

pub fn preamble(style: &Style) -> Vec<String> {
    let mut o = Vec::new();
    if let Some(bg) = &style.background {
        o.push(format!("style.fill: \"{bg}\""));
    }
    o.push("vars: {".into());
    for c in DIAGRAM {
        o.push(format!("  {}: \"{}\"", c.name, style.color(c)));
    }
    o.push("}".into());
    o.push(DIRECTION.into());
    o
}

pub fn write_and_render(
    out: &Out,
    stem: &str,
    lines: &[String],
    render_svg: bool,
    style: &Style,
) -> Result<()> {
    let d2_rel = PathBuf::from(format!("{stem}.d2"));
    out.write(&d2_rel, &lines.join("\n"))?;
    if !render_svg {
        return Ok(());
    }
    let svg_rel = PathBuf::from(format!("{stem}.svg"));
    let d2_path = out.root.join(&d2_rel);
    let svg_path = out.root.join(&svg_rel);
    let run = Command::new("d2")
        .args(D2_LAYOUT)
        .args(style.theme.d2())
        .arg(&d2_path)
        .arg(&svg_path)
        .output();
    match run {
        Err(e) if e.kind() == ErrorKind::NotFound => println!("{}", m::NO_D2),
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
        Ok(_) => {
            let svg = std::fs::read_to_string(&svg_path)?;
            std::fs::write(&svg_path, unmask(&svg))?;
            println!(
                "{}",
                fill(m::WROTE, &[("path", &svg_path.display().to_string())])
            );
        }
    }
    Ok(())
}
