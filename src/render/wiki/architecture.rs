//! The Architecture page: both diagrams, side by side, with the SVGs copied
//! next to the Markdown that references them.

use super::super::out::{Out, MD_MARKER};
use anyhow::Result;
use std::path::Path;

pub(super) fn page_architecture(out: &mut Out, src: &Path) -> Result<()> {
    for svg in ["topology.svg", "modules.svg"] {
        let from = out.root.join(svg);
        if from.exists() {
            let rel = src.join(svg);
            std::fs::create_dir_all(out.root.join(src))?;
            std::fs::copy(&from, out.root.join(&rel))?;
            out.record_svg(&rel);
        }
    }
    out.write_auto(
        &src.join("architecture.md"),
        &format!(
            "{MD_MARKER}\n\n\
             # Architecture\n\n\
             ## Data-flow topology\n\n\
             ![Data-flow topology](./topology.svg)\n\n\
             ## Module tree\n\n\
             ![Module tree](./modules.svg)\n"
        ),
    )
}
