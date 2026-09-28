use super::{diagram as inline, Page, Wiki};
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
        let body = [t::TOPOLOGY.into()].into_iter();
        Ok(Some(body.chain(inline(w, diagram::TOPOLOGY)).collect()))
    }
}
