//! d2 emission and SVG rendering.

use crate::output::Out;
use anyhow::Result;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

pub const D2_HEADER: [&str; 2] = [
    "# Auto-generated from the Nix config by nixdiag. Do not edit.",
    "# Regenerate: nixdiag gen",
];

pub fn write_and_render(
    out: &mut Out,
    stem: &str,
    lines: &[String],
    render_svg: bool,
) -> Result<()> {
    let d2_rel = PathBuf::from(format!("{stem}.d2"));
    out.write_auto(&d2_rel, &lines.join("\n"))?;
    if !render_svg {
        return Ok(());
    }
    let svg_rel = PathBuf::from(format!("{stem}.svg"));
    let d2_path = out.root.join(&d2_rel);
    let svg_path = out.root.join(&svg_rel);
    let run = Command::new("d2")
        .args(["--layout", "elk"])
        .arg(&d2_path)
        .arg(&svg_path)
        .output();
    match run {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            println!("(d2 binary not on PATH -- skipped SVG render)");
        }
        Err(e) => eprintln!("d2 render failed: {e}"),
        Ok(o) if !o.status.success() => {
            eprintln!("d2 render failed:\n{}", String::from_utf8_lossy(&o.stderr));
        }
        Ok(_) => {
            println!("wrote {}", svg_path.display());
            out.record_svg(&svg_rel);
        }
    }
    Ok(())
}
