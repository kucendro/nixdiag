//! mdBook wiki source — port of gen-wiki.py.
//!
//! One module per generated page; this file owns the options, the shared
//! host->services helper, and the order the pages are written in.

mod api;
mod architecture;
mod book;
mod closures;
mod endpoints;
mod hosts;
mod inputs;
mod services;

use api::page_api;
use architecture::page_architecture;
use book::{book_toml, copy_extra_pages, page_index, page_summary};
use closures::page_closures;
use endpoints::page_endpoints;
use hosts::page_hosts;
use inputs::page_inputs;
use services::page_services;

use super::d2::D2Style;
use super::out::Out;
use super::DocComments;
use crate::closures::Closures;
use crate::facts::{Facts, NixosHost};
use crate::source::annotations::Model;
use crate::source::flakelock::Lock;
use crate::source::repo::Repo;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Everything the pages draw on. Bundled because the data inputs keep
/// growing — facts, then annotations, then the lock, then closures — and a
/// positional argument list that long stops being readable.
pub struct WikiData<'a> {
    pub facts: &'a Facts,
    pub repo: &'a Repo,
    pub docs: &'a DocComments,
    pub model: &'a Model,
    pub lock: Option<&'a Lock>,
    pub closures: Option<&'a Closures>,
    /// Whether the `api/` tree was published, so the wiki can point at it
    /// without ever advertising endpoints that are not there.
    pub api: bool,
}

pub struct WikiOpts {
    pub title: String,
    /// (link title, source path) — appended to SUMMARY and copied into src/.
    pub extra_pages: Vec<(String, PathBuf)>,
    /// (link title, file name) — SUMMARY entry only, for pages some other
    /// tool writes into wiki/src itself.
    pub extra_links: Vec<(String, String)>,
}

/// Services this repo actually configures: name -> repo-relative files.
pub(super) fn repo_services(n: &NixosHost, repo: &Repo) -> BTreeMap<String, Vec<String>> {
    let mut svcs = BTreeMap::new();
    for item in &n.services {
        let files = repo.repo_files(&item.files);
        if !files.is_empty() {
            svcs.insert(item.name.clone(), files);
        }
    }
    svcs
}

/// `style` is passed alongside the data rather than folded into `WikiData`
/// because it is not data: the Closures page draws its own SVG chart and needs
/// the palette to do it.
pub fn generate(out: &mut Out, opts: &WikiOpts, style: &D2Style, d: &WikiData) -> Result<()> {
    let wiki = PathBuf::from("wiki");
    let src = wiki.join("src");

    book_toml(out, &wiki, &opts.title)?;
    let mut extra = copy_extra_pages(out, &src, &opts.extra_pages)?;
    extra.extend(opts.extra_links.iter().cloned());
    page_summary(
        out,
        &src,
        &extra,
        d.lock.is_some(),
        d.closures.is_some(),
        d.api,
    )?;
    page_index(out, &src)?;
    page_architecture(out, &src)?;
    page_hosts(out, &src, d.facts, d.repo, d.docs, d.closures)?;
    page_services(out, &src, d.facts, d.repo, d.docs)?;
    page_endpoints(out, &src, d.facts, d.model)?;
    if let Some(lock) = d.lock {
        page_inputs(out, &src, lock, style)?;
    }
    if let Some(closures) = d.closures {
        page_closures(out, &src, d.facts, closures, style)?;
    }
    if d.api {
        page_api(out, &src, d.lock.is_some(), d.closures.is_some())?;
    }
    Ok(())
}
