use super::Wiki;
use crate::conf::files::{page, BOOK};
use crate::text::fill;
use crate::text::messages::Fail;
use crate::text::wiki::{summary as t, INDEX};
use anyhow::{bail, Result};
use serde_json::json;
use std::path::PathBuf;

pub(super) fn book_toml(w: &Wiki, title: &str) -> Result<()> {
    let b = w.style.theme.book();
    let html = json!({
        "default-theme": b.default,
        "preferred-dark-theme": b.dark,
        "no-section-label": true,
    });
    let book = json!({ "book": { "title": title, "src": "src" }, "output": { "html": html } });
    w.out.write(BOOK, &toml::to_string(&book)?)
}

pub(super) fn copy_extra_pages(
    w: &Wiki,
    pages: &[(String, PathBuf)],
) -> Result<Vec<(String, String)>> {
    let mut links = Vec::new();
    for (title, source) in pages {
        let fname = source
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();
        if fname.is_empty() {
            bail!(Fail::ExtraPageNoName(title.clone()));
        }
        if !source.exists() {
            let (title, path) = (title.clone(), source.clone());
            bail!(Fail::ExtraPageMissing { title, path });
        }
        w.src.copy(source, &fname)?;
        links.push((title.clone(), fname));
    }
    Ok(links)
}

pub(super) fn page_summary(w: &Wiki, extra: &[(String, String)]) -> Result<()> {
    let mut entries = vec![t::FIXED.to_string()];
    if w.lock.is_some() {
        entries.push(t::INPUTS.into());
    }
    if w.closures.is_some() {
        entries.push(t::CLOSURES.into());
    }
    for (title, file) in extra {
        entries.push(fill(t::EXTRA, &[("title", title), ("file", file)]));
    }
    w.page(page::SUMMARY, &[t::TITLE.to_string(), entries.join("\n")])
}

pub(super) fn page_index(w: &Wiki) -> Result<()> {
    w.src.write(page::INDEX, INDEX)
}
