//! d2 emission and SVG rendering.

use crate::output::Out;
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

pub fn vars_block(style: &D2Style) -> Vec<String> {
    let mut o = vec!["vars: {".to_string()];
    for (name, light, dark) in PALETTE {
        let v = style
            .colors
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
            .unwrap_or(if style.dark { dark } else { light });
        o.push(format!("  {name}: \"{v}\""));
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
            println!("wrote {}", svg_path.display());
            out.record_svg(&svg_rel);
        }
    }
    Ok(())
}
