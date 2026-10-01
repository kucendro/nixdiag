mod architecture;
mod book;
mod closures;
mod endpoints;
mod firewall;
mod hosts;
mod inputs;
mod services;

use architecture::Architecture;
use book::{book_toml, copy_extra_pages, index, summary};
use closures::ClosuresPage;
use endpoints::Endpoints;
use firewall::FirewallPage;
use hosts::Hosts;
use inputs::Inputs;
use services::Services;

use super::out::Out;
use super::style::{Style, Theme};
use crate::closures::{Closures, Total};
use crate::conf::files::diagram::themed;
use crate::facts::{Facts, HostBase};
use crate::human::{Bytes, Count, Moment};
use crate::source::flakelock::Lock;
use crate::source::repo::Repo;
use crate::text::wiki::{self as text, code, NONE};
use crate::topology::Model;
use anyhow::Result;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::iter::once;
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
    pub generated: Option<i64>,
    pub revision: Option<String>,
}

pub trait Page {
    fn file(&self) -> &'static str;
    fn title(&self) -> &'static str;
    fn body(&self, w: &Wiki) -> Result<Option<Vec<String>>>;
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
    text::size_paths(Bytes(t.size), Count(t.paths))
}

fn codes<S: AsRef<str>>(items: impl IntoIterator<Item = S>, sep: &str) -> String {
    items.into_iter().map(|s| code(s.as_ref())).join(sep)
}

fn diagram(w: &Wiki, stem: &str) -> Option<String> {
    let [light, dark] = [Theme::Light, Theme::Dark].map(|t| themed(stem, t.name()));
    let exists = |f: &String| w.src.root.join(f).exists();
    (exists(&light) && exists(&dark)).then(|| text::diagram(&light, &dark))
}

pub(super) fn repo_services(b: &HostBase, repo: &Repo) -> BTreeMap<String, Vec<String>> {
    let files = b
        .services
        .iter()
        .map(|i| (i.name.clone(), repo.files(&i.files)));
    files.filter(|(_, f)| !f.is_empty()).collect()
}

pub fn generate(w: &Wiki, opts: &WikiOpts) -> Result<()> {
    book_toml(w, &opts.title)?;
    index(w)?;
    let pages: [&dyn Page; 7] = [
        &Architecture,
        &Hosts,
        &Services,
        &Endpoints,
        &FirewallPage,
        &Inputs,
        &ClosuresPage,
    ];
    let stamp = opts
        .generated
        .map(|at| text::generated(Moment(at), opts.revision.as_deref()));
    let mut listed = Vec::new();
    for p in pages {
        let Some(body) = p.body(w)? else { continue };
        let head = once(text::heading(p.title())).chain(stamp.clone());
        let page = head.chain(body).join("\n\n");
        w.src.write(p.file(), &page)?;
        listed.push((p.title().to_string(), p.file().to_string()));
    }
    listed.extend(copy_extra_pages(w, &opts.extra_pages)?);
    listed.extend(opts.extra_links.iter().cloned());
    summary(w, &listed)
}
