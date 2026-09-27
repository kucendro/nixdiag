use super::Wiki;
use crate::conf::files::{page, BOOK};
use crate::text::messages::Fail;
use crate::text::wiki::{summary as t, INDEX};
use anyhow::{bail, Result};
use serde_json::json;
use std::iter::once;
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

pub(super) fn summary(w: &Wiki, pages: &[(String, String)]) -> Result<()> {
    let entries = pages.iter().map(|(title, file)| t::entry(title, file));
    let entries: Vec<String> = once(t::entry(t::OVERVIEW, page::INDEX))
        .chain(entries)
        .collect();
    w.src.write(
        page::SUMMARY,
        &format!("{}\n\n{}", t::TITLE, entries.join("\n")),
    )
}

pub(super) fn index(w: &Wiki) -> Result<()> {
    w.src.write(page::INDEX, INDEX)
}
