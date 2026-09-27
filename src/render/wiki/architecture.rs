use super::Wiki;
use crate::conf::files::{diagram, page};
use crate::text::wiki::ARCHITECTURE;
use anyhow::Result;

pub(super) fn page_architecture(w: &Wiki) -> Result<()> {
    for stem in [diagram::TOPOLOGY, diagram::MODULES] {
        w.src.mirror(w.out, &format!("{stem}.svg"))?;
    }
    w.page(page::ARCHITECTURE, &[ARCHITECTURE.into()])
}
