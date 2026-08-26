//! facts -> full docs tree (topology + modules + wiki).

use crate::facts::{Facts, Host, SCHEMA};
use crate::output::Out;
use crate::repo::Repo;
use crate::wiki::WikiOpts;
use crate::{annotations, doccomment, modules, topology, wiki};
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Doc comments harvested from the repo, keyed for the wiki.
#[derive(Default)]
pub struct DocComments {
    /// host name -> doc of its entry module
    pub hosts: HashMap<String, String>,
    /// repo-relative module file -> its doc
    pub files: HashMap<String, String>,
}

fn collect_docs(facts: &Facts, repo: &Repo) -> DocComments {
    let mut docs = DocComments::default();
    let flake_text = std::fs::read_to_string(repo.root.join("flake.nix")).unwrap_or_default();
    for (host, f) in &facts.hosts {
        for entry in modules::host_entry_modules(host, &flake_text, repo) {
            if let Some(doc) = doccomment::from_file(&entry) {
                docs.hosts.insert(host.clone(), doc);
                break;
            }
        }
        let (services, programs) = match f {
            Host::Nixos(n) => (&n.services, &n.programs),
            Host::Darwin(d) => (&d.services, &d.programs),
        };
        for unit in services.iter().chain(programs) {
            for rel in repo.repo_files(&unit.files) {
                if let std::collections::hash_map::Entry::Vacant(e) = docs.files.entry(rel.clone())
                {
                    if let Some(doc) = doccomment::from_file(&repo.root.join(&rel)) {
                        e.insert(doc);
                    }
                }
            }
        }
    }
    docs
}

pub struct RenderOpts {
    pub repo: PathBuf,
    pub out: PathBuf,
    pub wiki: WikiOpts,
    pub svg: bool,
    pub style: crate::d2::D2Style,
    /// `@key` -> domain suffix for fqdn positions in annotations.
    pub domains: std::collections::BTreeMap<String, String>,
}

pub fn render_all(facts: &mut Facts, opts: &RenderOpts) -> Result<Out> {
    if facts.schema != SCHEMA {
        bail!(
            "facts schema {} does not match this nixdiag (expects {SCHEMA})",
            facts.schema
        );
    }
    facts.normalize();
    let repo = Repo::new(opts.repo.clone());
    let mut out = Out::new(opts.out.clone());

    let (model, diags) = annotations::collect(facts, &repo, &opts.domains);
    if !diags.is_empty() {
        for d in &diags {
            eprintln!("error: {d}");
        }
        bail!("{} annotation error(s)", diags.len());
    }
    if model.total == 0 {
        eprintln!(
            "note: no `#:` annotations found — the topology shows hosts and \
             firewall ports only. Annotate your modules to draw the data flow \
             (see the Annotations section of the nixdiag README)."
        );
    }

    topology::generate(facts, &model, &mut out, opts.svg, &opts.style)?;
    modules::generate(facts, &repo, &mut out, opts.svg, &opts.style)?;
    let docs = collect_docs(facts, &repo);
    wiki::generate(facts, &repo, &mut out, &opts.wiki, &docs, &model)?;
    Ok(out)
}
