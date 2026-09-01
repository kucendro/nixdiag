//! The Services page: every service this repo configures, which hosts enable
//! it, and the doc comment of the file that defines it.

use super::super::out::{Out, MD_MARKER};
use super::repo_services;
use crate::facts::Facts;
use crate::render::DocComments;
use crate::source::repo::Repo;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) fn page_services(
    out: &mut Out,
    src: &Path,
    facts: &Facts,
    repo: &Repo,
    docs: &DocComments,
) -> Result<()> {
    // name -> (hosts, files)
    let mut index: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
    for (host, f) in &facts.hosts {
        let Some(n) = f.as_nixos() else { continue };
        for (name, files) in repo_services(n, repo) {
            let e = index.entry(name).or_default();
            e.0.insert(host.clone());
            e.1.extend(files);
        }
    }
    let mut o: Vec<String> = vec![
        MD_MARKER.into(),
        "".into(),
        "# Services".into(),
        "".into(),
        "| Service | Hosts | Defined in |".into(),
        "|---|---|---|".into(),
    ];
    for (name, (hosts, files)) in &index {
        let hosts = hosts.iter().cloned().collect::<Vec<_>>().join(", ");
        let files = files
            .iter()
            .map(|x| format!("`{x}`"))
            .collect::<Vec<_>>()
            .join(" ");
        o.push(format!("| **{name}** | {hosts} | {files} |"));
    }
    if index.is_empty() {
        o.push("| — | — | — |".into());
    }
    // Doc-commented services get a section below the table.
    for (name, (_, files)) in &index {
        let Some(doc) = files.iter().find_map(|f| docs.files.get(f)) else {
            continue;
        };
        o.push("".into());
        o.push(format!("## {name}"));
        o.push("".into());
        o.push(doc.clone());
    }
    out.write_auto(&src.join("services.md"), &o.join("\n"))
}
