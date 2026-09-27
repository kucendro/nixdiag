mod args;

use crate::closures::Closures;
use crate::conf::schema;
use crate::facts::Facts;
use crate::render::{render_all, style::Style, RenderOpts, WikiOpts};
use crate::text::{fill, messages as m};
use anyhow::{bail, Context, Result};
use args::Cli;
use clap::Parser;
use serde::de::DeserializeOwned;
use std::path::Path;

pub fn run() -> Result<()> {
    let r = Cli::parse();
    let facts: Facts = read_json(&r.facts, m::FACTS)?;
    check(m::FACTS, m::FACTS_PRODUCER, facts.schema, schema::FACTS)?;
    let closures: Option<Closures> = r
        .closures
        .as_deref()
        .map(|p| read_json(p, m::CLOSURES))
        .transpose()?;
    if let Some(c) = &closures {
        check(
            m::CLOSURES,
            m::CLOSURES_PRODUCER,
            c.schema,
            schema::CLOSURES,
        )?;
    }
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
            style: Style {
                theme: r.theme,
                background: Some(r.background),
                colors: r.colors,
            },
            closures,
        },
    )
}

fn read_json<T: DeserializeOwned>(path: &Path, file: &str) -> Result<T> {
    let text = if path == Path::new("-") {
        std::io::read_to_string(std::io::stdin())?
    } else {
        std::fs::read_to_string(path)
            .with_context(|| fill(m::READING, &[("path", &path.display().to_string())]))?
    };
    serde_json::from_str(&text).with_context(|| fill(m::PARSING, &[("file", file)]))
}

fn check(file: &str, producer: &str, found: u32, expected: u32) -> Result<()> {
    if found != expected {
        bail!(fill(
            m::SCHEMA_MISMATCH,
            &[
                ("file", file),
                ("producer", producer),
                ("found", &found.to_string()),
                ("version", env!("CARGO_PKG_VERSION")),
                ("expected", &expected.to_string()),
            ],
        ));
    }
    Ok(())
}
