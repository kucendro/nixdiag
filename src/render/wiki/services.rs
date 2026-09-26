use super::super::out::Out;
use super::{page, repo_services};
use crate::facts::Facts;
use crate::source::repo::Repo;
use crate::text::fill;
use crate::text::wiki::services as t;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) fn page_services(out: &Out, src: &Path, facts: &Facts, repo: &Repo) -> Result<()> {
    let mut index: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
    for (host, f) in &facts.hosts {
        let Some(n) = f.as_nixos() else { continue };
        for (name, files) in repo_services(n, repo) {
            let e = index.entry(name).or_default();
            e.0.insert(host.clone());
            e.1.extend(files);
        }
    }
    let mut rows: Vec<String> = index
        .iter()
        .map(|(name, (hosts, files))| {
            let hosts = hosts.iter().cloned().collect::<Vec<_>>().join(", ");
            let files: Vec<String> = files
                .iter()
                .map(|f| fill(t::FILE, &[("file", f)]))
                .collect();
            fill(
                t::ROW,
                &[
                    ("name", name),
                    ("hosts", &hosts),
                    ("files", &files.join(" ")),
                ],
            )
        })
        .collect();
    if rows.is_empty() {
        rows.push(t::EMPTY.into());
    }
    let mut o = vec![
        t::TITLE.to_string(),
        fill(t::TABLE, &[("rows", &rows.join("\n"))]),
    ];
    let mut described: BTreeMap<&str, &str> = BTreeMap::new();
    for f in facts.hosts.values() {
        for (name, u) in &f.topology().units {
            if let Some(d) = &u.description {
                described.entry(name).or_insert(d);
            }
        }
    }
    for (name, description) in described {
        o.push(fill(
            t::UNIT,
            &[("name", name), ("description", description)],
        ));
    }
    page(out, &src.join("services.md"), &o)
}
