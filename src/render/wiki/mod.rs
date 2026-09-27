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
use crate::closures::{Closures, Total};
use crate::facts::{Facts, HostBase};
use crate::human::{Bytes, Count};
use crate::source::flakelock::Lock;
use crate::source::repo::Repo;
use crate::text::fill;
use crate::text::wiki::{CODE, NONE, SIZE_PATHS};
use crate::topology::Model;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;
use tabled::builder::Builder;
use tabled::settings::Style as Grid;

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

fn table<const N: usize>(head: [&str; N], rows: impl IntoIterator<Item = [String; N]>) -> String {
    let mut b = Builder::from_iter(rows);
    if b.count_records() == 0 {
        b.push_record([NONE; N]);
    }
    b.insert_record(0, head);
    b.build().with(Grid::markdown()).to_string()
}

fn size_paths(t: &Total) -> String {
    let (size, paths) = (Bytes(t.size).to_string(), Count(t.paths).to_string());
    fill(SIZE_PATHS, &[("size", &size), ("paths", &paths)])
}

fn code(s: &str) -> String {
    fill(CODE, &[("code", s)])
}

fn codes<S: AsRef<str>>(items: impl IntoIterator<Item = S>, sep: &str) -> String {
    let items: Vec<String> = items.into_iter().map(|s| code(s.as_ref())).collect();
    items.join(sep)
}

pub(super) fn repo_services(b: &HostBase, repo: &Repo) -> BTreeMap<String, Vec<String>> {
    let mut svcs = BTreeMap::new();
    for item in &b.services {
        let files = repo.files(&item.files);
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
