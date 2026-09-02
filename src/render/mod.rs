//! facts + annotations -> the full docs tree (topology, module tree, wiki).

mod api;
mod chart;
pub mod d2;
mod inputs;
mod modules;
pub mod out;
mod topology;
mod wiki;

pub use api::ApiOpts;
pub use out::{Out, WKind};
pub use wiki::WikiOpts;

use crate::closures::{Closures, CLOSURES_SCHEMA};
use crate::facts::{Facts, Host, SCHEMA};
use crate::source::annotations::{self, Sev};
use crate::source::flakelock::Lock;
use crate::source::repo::Repo;
use crate::source::{doccomment, imports};
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
    /// `@key` -> domain suffix for fqdn positions in annotations.
    pub domains: std::collections::BTreeMap<String, String>,
    /// Annotation grammar edition in force (already resolved against
    /// `annotations::GRAMMAR`).
    pub grammar: u32,
    /// Warning categories promoted to errors, e.g. `deprecated`.
    pub deny: Vec<String>,
    /// Per-host closure sizes, when `mkDocs { closures = true; }` supplied
    /// them. Absent in every other case, including all of mode A.
    pub closures: Option<Closures>,
    /// The published `api/` tree. `None` disables it entirely.
    pub api: Option<ApiOpts>,
}

pub fn render_all(facts: &mut Facts, opts: &RenderOpts) -> Result<Out> {
    if facts.schema != SCHEMA {
        // Both halves normally ship from one flake, so a mismatch means a
        // `lib` and a binary from different revisions — name the versions,
        // not just the numbers.
        bail!(
            "facts.json declares schema {}, but nixdiag {} implements schema {SCHEMA} — \
             the projection that produced these facts comes from a different nixdiag \
             revision; pin `lib` and the binary to the same one",
            facts.schema,
            env!("CARGO_PKG_VERSION")
        );
    }
    if let Some(c) = &opts.closures {
        if c.schema != CLOSURES_SCHEMA {
            bail!(
                "closures.json declares schema {}, but nixdiag {} implements schema \
                 {CLOSURES_SCHEMA} — the derivation that produced it comes from a \
                 different nixdiag revision; pin `lib` and the binary to the same one",
                c.schema,
                env!("CARGO_PKG_VERSION")
            );
        }
    }
    facts.normalize();
    let repo = Repo::new(opts.repo.clone());
    let mut out = Out::new(opts.out.clone());

    let (model, diags) = annotations::collect(facts, &repo, &opts.domains, opts.grammar);
    let deny_deprecated = opts.deny.iter().any(|d| d == "deprecated");
    let mut errors = 0;
    for d in &diags {
        if d.sev == Sev::Error || deny_deprecated {
            errors += 1;
            eprintln!("error: {d}");
        } else {
            eprintln!("warning: {d}");
        }
    }
    if errors > 0 {
        bail!("{errors} annotation error(s)");
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
    // A flake without a lock is legitimate; the input pages are simply absent.
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
    )?;
    if let Some(api_opts) = &opts.api {
        api::generate(
            &mut out,
            api_opts,
            &api::ApiData {
                facts,
                repo: &repo,
                model: &model,
                lock: lock.as_ref(),
                closures: opts.closures.as_ref(),
            },
        )?;
    }
    Ok(out)
}
