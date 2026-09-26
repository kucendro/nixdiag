mod chart;
pub mod d2;
mod inputs;
mod modules;
pub mod out;
mod topology;
mod unmask;
mod wiki;

pub use out::Out;
pub use wiki::WikiOpts;

use crate::closures::{Closures, CLOSURES_SCHEMA};
use crate::facts::{Facts, Host, SCHEMA};
use crate::source::annotations::{self, Sev};
use crate::source::flakelock::Lock;
use crate::source::repo::Repo;
use crate::source::{doccomment, imports};
use crate::text::{fill, messages as m};
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default)]
pub struct DocComments {
    pub hosts: HashMap<String, String>,
    pub files: HashMap<String, String>,
}

fn collect_docs(facts: &Facts, repo: &Repo) -> DocComments {
    let mut docs = DocComments::default();
    let flake_text = std::fs::read_to_string(repo.root.join("flake.nix")).unwrap_or_default();
    for (host, f) in &facts.hosts {
        for entry in imports::host_entry_modules(host, &flake_text, repo) {
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
    pub style: d2::D2Style,
    pub domains: std::collections::BTreeMap<String, String>,
    pub grammar: u32,
    pub deny: Vec<String>,
    pub closures: Option<Closures>,
}

pub fn render_all(facts: &mut Facts, opts: &RenderOpts) -> Result<()> {
    let mismatch = |template: &str, found: u32, expected: u32| {
        fill(
            template,
            &[
                ("found", &found.to_string()),
                ("version", env!("CARGO_PKG_VERSION")),
                ("expected", &expected.to_string()),
            ],
        )
    };
    if facts.schema != SCHEMA {
        bail!(mismatch(m::SCHEMA_MISMATCH, facts.schema, SCHEMA));
    }
    if let Some(c) = &opts.closures {
        if c.schema != CLOSURES_SCHEMA {
            bail!(mismatch(
                m::CLOSURES_SCHEMA_MISMATCH,
                c.schema,
                CLOSURES_SCHEMA
            ));
        }
    }
    facts.normalize();
    let repo = Repo::new(opts.repo.clone());
    let mut out = Out::new(opts.out.clone());

    let (model, diags) = annotations::collect(facts, &repo, &opts.domains, opts.grammar);
    let deny_deprecated = opts.deny.iter().any(|d| d == "deprecated");
    let mut errors = 0;
    for d in &diags {
        let template = if d.sev == Sev::Error || deny_deprecated {
            errors += 1;
            m::ERROR
        } else {
            m::WARNING
        };
        eprintln!("{}", fill(template, &[("message", &d.to_string())]));
    }
    if errors > 0 {
        bail!(fill(
            m::ANNOTATION_ERRORS,
            &[("count", &errors.to_string())]
        ));
    }
    if model.total == 0 {
        eprintln!("{}", m::NO_ANNOTATIONS);
    }

    topology::generate(facts, &model, &mut out, opts.svg, &opts.style)?;
    modules::generate(facts, &repo, &mut out, opts.svg, &opts.style)?;
    let lock = Lock::read(&repo.root);
    if let Some(lock) = &lock {
        inputs::generate(lock, &mut out, opts.svg, &opts.style)?;
    }
    let docs = collect_docs(facts, &repo);
    wiki::generate(
        &mut out,
        &opts.wiki,
        &opts.style,
        &wiki::WikiData {
            facts,
            repo: &repo,
            docs: &docs,
            model: &model,
            lock: lock.as_ref(),
            closures: opts.closures.as_ref(),
        },
    )
}
