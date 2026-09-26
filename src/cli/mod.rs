mod options;

use crate::closures::Closures;
use crate::facts::Facts;
use crate::render::render_all;
use crate::source::annotations;
use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "nixdiag",
    version = annotations::VERSION,
    about = "Static infrastructure docs from any Nix flake"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Args)]
pub struct RenderArgs {
    /// facts.json path, or - for stdin
    #[arg(long)]
    facts: PathBuf,
    /// Repo source the facts refer to
    #[arg(long, default_value = ".")]
    repo: PathBuf,
    /// closures.json from `mkDocs { closures = true; }`, adding the
    /// Closures page
    #[arg(long)]
    closures: Option<PathBuf>,
    #[arg(long, default_value = "docs")]
    out: PathBuf,
    /// Wiki title (used only when seeding book.toml)
    #[arg(long)]
    title: Option<String>,
    /// Extra hand-written wiki page as TITLE=FILE; repeatable
    #[arg(long = "extra-page", value_name = "TITLE=FILE")]
    extra_pages: Vec<String>,
    /// SUMMARY entry as TITLE=NAME.md for a page written into wiki/src by
    /// another tool; repeatable
    #[arg(long = "extra-link", value_name = "TITLE=NAME.md")]
    extra_links: Vec<String>,
    /// Skip SVG rendering (d2)
    #[arg(long)]
    no_svg: bool,
    /// Color theme: dark (default) or light
    #[arg(long, value_parser = ["light", "dark"])]
    theme: Option<String>,
    /// Diagram canvas fill (default transparent)
    #[arg(long)]
    background: Option<String>,
    /// Palette override as NAME=#HEX (names: the vars block in the d2 output,
    /// plus chartShared/chartPartial/chartUnique/chartInk/chartMuted/
    /// chartTrack for the SVG charts); repeatable
    #[arg(long = "color", value_name = "NAME=#HEX")]
    colors: Vec<String>,
    /// Domain suffix for `@KEY` in annotation fqdns, as KEY=DOMAIN;
    /// repeatable
    #[arg(long = "domain", value_name = "KEY=DOMAIN")]
    domains: Vec<String>,
    /// Annotation grammar edition the repo is written against (default: the
    /// edition this binary implements)
    #[arg(long, value_name = "N")]
    grammar: Option<u32>,
    /// Promote a warning category to an error; repeatable
    #[arg(long = "deny", value_name = "CATEGORY", value_parser = ["deprecated"])]
    deny: Vec<String>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Render docs from facts.json (needs the repo source, not nix)
    Render(Box<RenderArgs>),
    /// Print the annotation cheat sheet (SYNTAX.md) this binary parses
    Syntax,
}

pub fn run() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Render(r) => {
            let mut facts: Facts = read_json(&r.facts).context("parsing facts.json")?;
            let closures: Option<Closures> = r
                .closures
                .as_deref()
                .map(|p| read_json(p).context("parsing closures.json"))
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
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?
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
