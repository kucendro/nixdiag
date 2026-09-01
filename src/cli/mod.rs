//! Command-line surface: the clap types and the dispatch into `commands`.

mod commands;
mod options;

use commands::{cmd_check, cmd_facts, cmd_gen, cmd_render};

use crate::source::annotations;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

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
pub struct FlakeArgs {
    /// Flake directory to evaluate
    #[arg(long, default_value = ".")]
    flake: PathBuf,
    /// Restrict to these hosts (default: all discovered)
    hosts: Vec<String>,
}

#[derive(Args)]
pub struct RenderArgs {
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
    /// Evaluate flake configurations into facts.json on stdout
    Facts(FlakeArgs),
    /// Render docs from an existing facts.json (needs the repo source, not nix)
    Render {
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
        #[command(flatten)]
        render: RenderArgs,
    },
    /// facts + render in one step
    Gen {
        #[command(flatten)]
        flake: FlakeArgs,
        /// Output directory (default: <flake>/docs)
        #[arg(long)]
        out: Option<PathBuf>,
        /// Not available in mode A — see the error it prints
        #[arg(long)]
        closures: bool,
        #[command(flatten)]
        render: RenderArgs,
    },
    /// Regenerate to a temp dir and diff against committed docs (CI gate)
    Check {
        #[command(flatten)]
        flake: FlakeArgs,
        /// Committed docs directory to compare against (default: <flake>/docs)
        #[arg(long)]
        out: Option<PathBuf>,
        /// Not available in mode A — see the error it prints
        #[arg(long)]
        closures: bool,
        #[command(flatten)]
        render: RenderArgs,
    },
}

/// Parse argv and run the requested subcommand.
pub fn run() -> anyhow::Result<()> {
    match Cli::parse().cmd {
        Cmd::Facts(f) => cmd_facts(f),
        Cmd::Render {
            facts,
            repo,
            out,
            closures,
            render,
        } => cmd_render(facts, repo, out, closures, render),
        Cmd::Gen {
            flake,
            out,
            closures,
            render,
        } => cmd_gen(flake, out, closures, render).map(|_| ()),
        Cmd::Check {
            flake,
            out,
            closures,
            render,
        } => cmd_check(flake, out, closures, render),
    }
}
