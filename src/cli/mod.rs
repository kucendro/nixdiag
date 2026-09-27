mod args;

use crate::closures::Closures;
use crate::facts::Facts;
use crate::render::{d2::D2Style, render_all, RenderOpts, WikiOpts};
use crate::text::{fill, messages as m};
use anyhow::{Context, Result};
use args::Cli;
use clap::Parser;
use serde::de::DeserializeOwned;
use std::path::Path;

pub fn run() -> Result<()> {
    let r = Cli::parse();
    let facts: Facts = read_json(&r.facts).context(m::PARSING_FACTS)?;
    let closures: Option<Closures> = r
        .closures
        .as_deref()
        .map(|p| read_json(p).context(m::PARSING_CLOSURES))
        .transpose()?;
    render_all(
        &facts,
        &RenderOpts {
            repo: r.repo,
            out: r.out,
            wiki: WikiOpts {
                title: r.title,
                extra_pages: r.extra_pages,
                extra_links: r.extra_links,
            },
            svg: !r.no_svg,
            style: D2Style {
                dark: r.theme == "dark",
                background: Some(r.background),
                colors: r.colors,
            },
            closures,
        },
    )
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let text = if path == Path::new("-") {
        std::io::read_to_string(std::io::stdin())?
    } else {
        std::fs::read_to_string(path)
            .with_context(|| fill(m::READING, &[("path", &path.display().to_string())]))?
    };
    Ok(serde_json::from_str(&text)?)
}
