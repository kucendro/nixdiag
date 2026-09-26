mod options;

use crate::closures::Closures;
use crate::facts::Facts;
use crate::render::render_all;
use crate::source::annotations;
use crate::text::{cli as t, fill, messages as m};
use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "nixdiag", version = annotations::VERSION, about = t::ABOUT)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Args)]
pub struct RenderArgs {
    #[arg(long, help = t::FACTS)]
    facts: PathBuf,
    #[arg(long, default_value = ".", help = t::REPO)]
    repo: PathBuf,
    #[arg(long, help = t::CLOSURES)]
    closures: Option<PathBuf>,
    #[arg(long, default_value = "docs", help = t::OUT)]
    out: PathBuf,
    #[arg(long, help = t::TITLE)]
    title: Option<String>,
    #[arg(long = "extra-page", value_name = "TITLE=FILE", help = t::EXTRA_PAGE)]
    extra_pages: Vec<String>,
    #[arg(long = "extra-link", value_name = "TITLE=NAME.md", help = t::EXTRA_LINK)]
    extra_links: Vec<String>,
    #[arg(long, help = t::NO_SVG)]
    no_svg: bool,
    #[arg(long, value_parser = ["light", "dark"], help = t::THEME)]
    theme: Option<String>,
    #[arg(long, help = t::BACKGROUND)]
    background: Option<String>,
    #[arg(long = "color", value_name = "NAME=#HEX", help = t::COLOR)]
    colors: Vec<String>,
    #[arg(long = "domain", value_name = "KEY=DOMAIN", help = t::DOMAIN)]
    domains: Vec<String>,
    #[arg(long, value_name = "N", help = t::GRAMMAR)]
    grammar: Option<u32>,
    #[arg(long = "deny", value_name = "CATEGORY", value_parser = ["deprecated"], help = t::DENY)]
    deny: Vec<String>,
}

#[derive(Subcommand)]
enum Cmd {
    #[command(about = t::RENDER)]
    Render(Box<RenderArgs>),
    #[command(about = t::SYNTAX)]
    Syntax,
}

pub fn run() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Render(r) => {
            let mut facts: Facts = read_json(&r.facts).context(m::PARSING_FACTS)?;
            let closures: Option<Closures> = r
                .closures
                .as_deref()
                .map(|p| read_json(p).context(m::PARSING_CLOSURES))
                .transpose()?;
            render_all(&mut facts, &options::to_render_opts(&r, closures)?)
        }
        Cmd::Syntax => {
            print!("{}", include_str!("../../SYNTAX.md"));
            Ok(())
        }
    }
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

fn abs(p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(p)
    }
}
