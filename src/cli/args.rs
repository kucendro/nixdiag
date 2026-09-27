use crate::render::d2::PALETTE;
use crate::text::{cli as t, fill, messages as m};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nixdiag", version, about = t::ABOUT)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    #[command(about = t::RENDER)]
    Render(RenderArgs),
}

#[derive(Args)]
pub struct RenderArgs {
    #[arg(long, help = t::FACTS)]
    pub facts: PathBuf,
    #[arg(long, default_value = ".", value_parser = absolute, help = t::REPO)]
    pub repo: PathBuf,
    #[arg(long, help = t::CLOSURES)]
    pub closures: Option<PathBuf>,
    #[arg(long, default_value = "docs", value_parser = absolute, help = t::OUT)]
    pub out: PathBuf,
    #[arg(long, default_value = t::DEFAULT_TITLE, help = t::TITLE)]
    pub title: String,
    #[arg(long = "extra-page", value_name = "TITLE=FILE", value_parser = pair::<PathBuf>, help = t::EXTRA_PAGE)]
    pub extra_pages: Vec<(String, PathBuf)>,
    #[arg(long = "extra-link", value_name = "TITLE=NAME.md", value_parser = pair::<String>, help = t::EXTRA_LINK)]
    pub extra_links: Vec<(String, String)>,
    #[arg(long, help = t::NO_SVG)]
    pub no_svg: bool,
    #[arg(long, default_value = "dark", value_parser = ["dark", "light"], help = t::THEME)]
    pub theme: String,
    #[arg(long, default_value = "transparent", help = t::BACKGROUND)]
    pub background: String,
    #[arg(long = "color", value_name = "NAME=#HEX", value_parser = color, help = t::COLOR)]
    pub colors: Vec<(String, String)>,
}

fn absolute(s: &str) -> std::io::Result<PathBuf> {
    std::path::absolute(s)
}

fn pair<V: From<String>>(s: &str) -> Result<(String, V), String> {
    match s.split_once('=') {
        Some((k, v)) if !k.is_empty() && !v.is_empty() => Ok((k.into(), V::from(v.into()))),
        _ => Err(m::BAD_PAIR.into()),
    }
}

fn color(s: &str) -> Result<(String, String), String> {
    let (name, hex) = pair::<String>(s)?;
    if PALETTE.iter().any(|(p, ..)| *p == name) {
        return Ok((name, hex));
    }
    let known: Vec<&str> = PALETTE.iter().map(|(p, ..)| *p).collect();
    Err(fill(
        m::UNKNOWN_COLOR,
        &[("name", &name), ("palette", &known.join(", "))],
    ))
}
