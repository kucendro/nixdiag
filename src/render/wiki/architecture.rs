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
        w.src.mirror(w.out, &format!("{}.svg", diagram::TOPOLOGY))?;
        Ok(Some(vec![t::BODY.into()]))
    }
}
