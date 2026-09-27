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

use super::out::Out;
use super::style::Style;
use crate::closures::Closures;
use crate::facts::{Facts, HostBase};
use crate::source::flakelock::Lock;
use crate::source::repo::Repo;
use crate::topology::Model;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub struct Wiki<'a> {
    pub out: &'a Out,
    pub src: Out,
    pub style: &'a Style,
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

impl Wiki<'_> {
    fn page(&self, name: &str, sections: &[String]) -> Result<()> {
        self.src.write(name, &sections.join("\n\n"))
    }
}

pub(super) fn repo_services(b: &HostBase, repo: &Repo) -> BTreeMap<String, Vec<String>> {
    let mut svcs = BTreeMap::new();
    for item in &b.services {
        let files = repo.repo_files(&item.files);
        if !files.is_empty() {
            svcs.insert(item.name.clone(), files);
        }
    }
    svcs
}

pub fn generate(w: &Wiki, opts: &WikiOpts) -> Result<()> {
    book_toml(w, &opts.title)?;
    let mut extra = copy_extra_pages(w, &opts.extra_pages)?;
    extra.extend(opts.extra_links.iter().cloned());
    page_summary(w, &extra)?;
    page_index(w)?;
    page_architecture(w)?;
    page_hosts(w)?;
    page_services(w)?;
    page_endpoints(w)?;
    if let Some(lock) = w.lock {
        page_inputs(w, lock)?;
    }
    if let Some(closures) = w.closures {
        page_closures(w, closures)?;
    }
    Ok(())
}
