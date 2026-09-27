pub mod chart;
pub mod dot;
mod inputs;
mod modules;
pub mod out;
pub mod style;
mod topology;
mod wiki;

pub use out::Out;
pub use wiki::WikiOpts;

use crate::closures::Closures;
use crate::conf::files;
use crate::facts::Facts;
use crate::source::flakelock::Lock;
use crate::source::repo::Repo;
use crate::text::messages as m;
use anyhow::Result;
use std::path::PathBuf;
use style::Style;

pub struct RenderOpts {
    pub repo: PathBuf,
    pub out: PathBuf,
    pub wiki: WikiOpts,
    pub svg: bool,
    pub style: Style,
    pub closures: Option<Closures>,
}

pub fn render_all(facts: &Facts, opts: &RenderOpts) -> Result<()> {
    let repo = Repo::new(opts.repo.clone());
    let out = Out::new(opts.out.clone());

    let model = crate::topology::build(facts)?;
    if facts.bare() {
        eprintln!("{}", m::NO_TOPOLOGY);
    }

    let dot = dot::Dot {
        out: &out,
        style: &opts.style,
        svg: opts.svg,
    };
    dot.render(&topology::Topology::new(facts, &model))?;
    dot.render(&modules::Modules::new(facts, &repo)?)?;
    let lock = Lock::read(&repo.root);
    if let Some(lock) = &lock {
        dot.render(&inputs::Inputs(lock))?;
    }
    wiki::generate(
        &wiki::Wiki {
            out: &out,
            src: out.sub(files::SRC),
            style: &opts.style,
            facts,
            repo: &repo,
            model: &model,
            lock: lock.as_ref(),
            closures: opts.closures.as_ref(),
        },
        &opts.wiki,
    )
}
