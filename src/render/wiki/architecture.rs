use super::{Page, Wiki};
use crate::conf::files::{diagram, page};
use crate::text::wiki::architecture as t;
use anyhow::Result;

pub(super) struct Architecture;

impl Page for Architecture {
    fn file(&self) -> &'static str {
        page::ARCHITECTURE
    }

    fn title(&self) -> &'static str {
        t::TITLE
    }

    fn body(&self, w: &Wiki) -> Result<Option<Vec<String>>> {
        for stem in [diagram::TOPOLOGY, diagram::MODULES] {
            w.src.mirror(w.out, &format!("{stem}.svg"))?;
        }
        Ok(Some(vec![t::BODY.into()]))
    }
}
