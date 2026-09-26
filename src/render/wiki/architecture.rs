use super::super::out::Out;
use super::page;
use crate::text::wiki::ARCHITECTURE;
use anyhow::Result;
use std::path::Path;

pub(super) fn page_architecture(out: &mut Out, src: &Path) -> Result<()> {
    for svg in ["topology.svg", "modules.svg"] {
        let from = out.root.join(svg);
        if from.exists() {
            let rel = src.join(svg);
            std::fs::create_dir_all(out.root.join(src))?;
            std::fs::copy(&from, out.root.join(&rel))?;
        }
    }
    page(out, &src.join("architecture.md"), &[ARCHITECTURE.into()])
}
