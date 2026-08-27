//! mdBook wiki source — port of gen-wiki.py.
//!
//! One module per generated page; this file owns the options, the shared
//! host->services helper, and the order the pages are written in.

mod architecture;
mod book;
mod endpoints;
mod hosts;
mod services;

use architecture::page_architecture;
use book::{book_toml, copy_extra_pages, page_index, page_summary};
use endpoints::page_endpoints;
use hosts::page_hosts;
use services::page_services;

use super::out::Out;
use super::DocComments;
use crate::facts::{Facts, NixosHost};
use crate::source::annotations::Model;
use crate::source::repo::Repo;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;

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

pub fn generate(
    facts: &Facts,
    repo: &Repo,
    out: &mut Out,
    opts: &WikiOpts,
    docs: &DocComments,
    model: &Model,
) -> Result<()> {
    let wiki = PathBuf::from("wiki");
    let src = wiki.join("src");

    book_toml(out, &wiki, &opts.title)?;
    let mut extra = copy_extra_pages(out, &src, &opts.extra_pages)?;
    extra.extend(opts.extra_links.iter().cloned());
    page_summary(out, &src, &extra)?;
    page_index(out, &src)?;
    page_architecture(out, &src)?;
    page_hosts(out, &src, facts, repo, docs)?;
    page_services(out, &src, facts, repo, docs)?;
    page_endpoints(out, &src, facts, model)?;
    Ok(())
}
