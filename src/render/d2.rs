use super::unmask::unmask;
use crate::render::out::Out;
use crate::text::d2::DIRECTION;
use crate::text::{fill, messages as m};
use anyhow::{bail, Result};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default)]
pub struct D2Style {
    pub dark: bool,
    pub background: Option<String>,
    pub colors: Vec<(String, String)>,
}

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

pub fn preamble(style: &D2Style) -> Vec<String> {
    let mut o = Vec::new();
    if let Some(bg) = &style.background {
        o.push(format!("style.fill: \"{bg}\""));
    }
    o.push("vars: {".into());
    for (name, light, dark) in PALETTE {
        o.push(format!(
            "  {name}: \"{}\"",
            color(style, name, (light, dark))
        ));
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
    style: &D2Style,
) -> Result<()> {
    let d2_rel = PathBuf::from(format!("{stem}.d2"));
    out.write(&d2_rel, &lines.join("\n"))?;
    if !render_svg {
        return Ok(());
    }
    let svg_rel = PathBuf::from(format!("{stem}.svg"));
    let d2_path = out.root.join(&d2_rel);
    let svg_path = out.root.join(&svg_rel);
    let mut cmd = Command::new("d2");
    cmd.args(["--layout", "elk"]);
    if style.dark {
        cmd.args(["--theme", "200"]);
    }
    let run = cmd.arg(&d2_path).arg(&svg_path).output();
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
