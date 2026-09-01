//! mdBook scaffolding: the book config, the SUMMARY index, the write-once
//! landing page, and hand-written pages copied in from outside.

use super::super::out::{Out, MD_MARKER};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub(super) fn book_toml(out: &mut Out, wiki: &Path, title: &str) -> Result<()> {
    out.write_once(
        &wiki.join("book.toml"),
        &format!(
            "[book]\n\
             title = \"{title}\"\n\
             src = \"src\"\n\n\
             [output.html]\n\
             default-theme = \"navy\"\n\
             preferred-dark-theme = \"navy\"\n\
             no-section-label = true\n"
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
        out.record_extra(&dest_rel);
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
    let mut text = format!(
        "{MD_MARKER}\n\n\
         # Summary\n\n\
         - [Overview](./index.md)\n\
         - [Architecture](./architecture.md)\n\
         - [Hosts](./hosts.md)\n\
         - [Services](./services.md)\n\
         - [Endpoints](./endpoints.md)\n"
    );
    if has_inputs {
        text.push_str("- [Inputs](./inputs.md)\n");
    }
    if has_closures {
        text.push_str("- [Closures](./closures.md)\n");
    }
    for (title, fname) in extra {
        text.push_str(&format!("- [{title}](./{fname})\n"));
    }
    out.write_auto(&src.join("SUMMARY.md"), &text)
}

pub(super) fn page_index(out: &mut Out, src: &Path) -> Result<()> {
    out.write_once(
        &src.join("index.md"),
        "# Infrastructure wiki\n\n\
         _Hand-written overview goes here_ — the big picture, where a newcomer \
         should start, and *why* things are the way they are.\n\n\
         Everything else in this wiki (Architecture, Hosts, Services, Endpoints) \
         is **auto-generated from the Nix configuration**, so it is always \
         current. This page is the one you edit by hand.\n",
    )
}
