use super::Wiki;
use crate::conf::files::{page, BOOK as BOOK_FILE};
use crate::text::wiki::{summary as t, BOOK, INDEX};
use crate::text::{fill, messages as m};
use anyhow::{bail, Result};
use std::path::PathBuf;

pub(super) fn book_toml(w: &Wiki, title: &str) -> Result<()> {
    let b = w.style.theme.book();
    w.out.write(
        BOOK_FILE,
        &fill(
            BOOK,
            &[("title", title), ("default", b.default), ("dark", b.dark)],
        ),
    )
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
            bail!(fill(m::EXTRA_PAGE_NO_NAME, &[("title", title)]));
        }
        if !source.exists() {
            bail!(fill(
                m::EXTRA_PAGE_MISSING,
                &[("title", title), ("path", &source.display().to_string())]
            ));
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
