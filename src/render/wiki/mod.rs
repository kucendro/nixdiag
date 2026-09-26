mod architecture;
mod book;
mod closures;
mod endpoints;
mod hosts;
mod inputs;
mod services;

use architecture::page_architecture;
use book::{book_toml, copy_extra_pages, page_index, page_summary};
use closures::page_closures;
use endpoints::page_endpoints;
use hosts::page_hosts;
use inputs::page_inputs;
use services::page_services;

use super::d2::D2Style;
use super::out::{Out, MD_MARKER};
use crate::closures::Closures;
use crate::facts::{Facts, NixosHost};
use crate::source::flakelock::Lock;
use crate::source::repo::Repo;
use crate::topology::Model;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct WikiData<'a> {
    pub facts: &'a Facts,
    pub repo: &'a Repo,
    pub model: &'a Model,
    pub lock: Option<&'a Lock>,
    pub closures: Option<&'a Closures>,
}

pub struct WikiOpts {
    pub title: String,
    pub extra_pages: Vec<(String, PathBuf)>,
    pub extra_links: Vec<(String, String)>,
}

pub(super) fn page(out: &mut Out, rel: &Path, sections: &[String]) -> Result<()> {
    out.write_auto(rel, &format!("{MD_MARKER}\n\n{}", sections.join("\n\n")))
}

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

pub fn generate(out: &mut Out, opts: &WikiOpts, style: &D2Style, d: &WikiData) -> Result<()> {
    let wiki = PathBuf::from("wiki");
    let src = wiki.join("src");

    book_toml(out, &wiki, &opts.title, style.dark)?;
    let mut extra = copy_extra_pages(out, &src, &opts.extra_pages)?;
    extra.extend(opts.extra_links.iter().cloned());
    page_summary(out, &src, &extra, d.lock.is_some(), d.closures.is_some())?;
    page_index(out, &src)?;
    page_architecture(out, &src)?;
    page_hosts(out, &src, d.facts, d.repo, d.closures)?;
    page_services(out, &src, d.facts, d.repo)?;
    page_endpoints(out, &src, d.facts, d.model)?;
    if let Some(lock) = d.lock {
        page_inputs(out, &src, lock, style)?;
    }
    if let Some(closures) = d.closures {
        page_closures(out, &src, d.facts, closures, style)?;
    }
    Ok(())
}
