use super::super::out::Out;
use super::page;
use crate::text::fill;
use crate::text::wiki::{summary as t, BOOK, INDEX};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub(super) fn book_toml(out: &mut Out, wiki: &Path, title: &str, dark: bool) -> Result<()> {
    let (default, preferred_dark) = if dark {
        ("navy", "navy")
    } else {
        ("light", "coal")
    };
    out.write_once(
        &wiki.join("book.toml"),
        &fill(
            BOOK,
            &[
                ("title", title),
                ("default", default),
                ("dark", preferred_dark),
            ],
        ),
    )
}

pub(super) fn copy_extra_pages(
    out: &mut Out,
    src: &Path,
    pages: &[(String, PathBuf)],
) -> Result<Vec<(String, String)>> {
    let mut links = Vec::new();
    for (title, source) in pages {
        let fname = source
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();
        if fname.is_empty() {
            bail!("--extra-page {title}: source has no file name");
        }
        let dest_rel = src.join(&fname);
        let dest = out.root.join(&dest_rel);
        if !source.exists() {
            bail!("--extra-page {title}: {} not found", source.display());
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(source, &dest)?;
        println!("wrote {}", dest.display());
        links.push((title.clone(), fname));
    }
    Ok(links)
}

pub(super) fn page_summary(
    out: &mut Out,
    src: &Path,
    extra: &[(String, String)],
    has_inputs: bool,
    has_closures: bool,
) -> Result<()> {
    let mut entries = vec![t::FIXED.to_string()];
    if has_inputs {
        entries.push(t::INPUTS.into());
    }
    if has_closures {
        entries.push(t::CLOSURES.into());
    }
    for (title, file) in extra {
        entries.push(fill(t::EXTRA, &[("title", title), ("file", file)]));
    }
    page(
        out,
        &src.join("SUMMARY.md"),
        &[t::TITLE.to_string(), entries.join("\n")],
    )
}

pub(super) fn page_index(out: &mut Out, src: &Path) -> Result<()> {
    out.write_once(&src.join("index.md"), INDEX)
}
