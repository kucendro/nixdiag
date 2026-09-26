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
use crate::facts::{Facts, SCHEMA};
use crate::source::flakelock::Lock;
use crate::source::repo::Repo;
use crate::text::{fill, messages as m};
use anyhow::{bail, Result};
use std::path::PathBuf;

pub struct RenderOpts {
    pub repo: PathBuf,
    pub out: PathBuf,
    pub wiki: WikiOpts,
    pub svg: bool,
    pub style: d2::D2Style,
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

    let model = crate::topology::build(facts)?;
    if facts.bare() {
        eprintln!("{}", m::NO_TOPOLOGY);
    }

    topology::generate(facts, &model, &mut out, opts.svg, &opts.style)?;
    modules::generate(facts, &repo, &mut out, opts.svg, &opts.style)?;
    let lock = Lock::read(&repo.root);
    if let Some(lock) = &lock {
        inputs::generate(lock, &mut out, opts.svg, &opts.style)?;
    }
    wiki::generate(
        &mut out,
        &opts.wiki,
        &opts.style,
        &wiki::WikiData {
            facts,
            repo: &repo,
            model: &model,
            lock: lock.as_ref(),
            closures: opts.closures.as_ref(),
        },
    )
}
