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
        let sections = [
            (t::TOPOLOGY, diagram::TOPOLOGY),
            (t::NETWORKS, diagram::NETWORKS),
            (t::OVERLAY, diagram::OVERLAY),
        ];
        let boards = sections
            .into_iter()
            .filter_map(|(title, stem)| Some([title.into(), inline(w, stem)?]));
        Ok(Some(boards.flatten().collect()))
    }
}
