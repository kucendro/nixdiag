use crate::conf::palette::{chart, DIAGRAM};
use crate::render::style::Theme;
use crate::text::cli as t;
use crate::text::messages::BadArg;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nixdiag", version, about = t::ABOUT)]
pub struct Cli {
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
    #[arg(long, value_enum, default_value_t, help = t::THEME)]
    pub theme: Theme,
    #[arg(long, default_value = "transparent", help = t::BACKGROUND)]
    pub background: String,
    #[arg(long = "color", value_name = "NAME=#HEX", value_parser = color, help = t::COLOR)]
    pub colors: Vec<(String, String)>,
}

fn absolute(s: &str) -> std::io::Result<PathBuf> {
    std::path::absolute(s)
}

fn pair<V: From<String>>(s: &str) -> Result<(String, V), BadArg> {
    match s.split_once('=') {
        Some((k, v)) if !k.is_empty() && !v.is_empty() => Ok((k.into(), V::from(v.into()))),
        _ => Err(BadArg::Pair),
    }
}

fn color(s: &str) -> Result<(String, String), BadArg> {
    let (name, hex) = pair::<String>(s)?;
    let known: Vec<&str> = DIAGRAM.iter().chain(chart::ALL).map(|c| c.name).collect();
    if known.contains(&name.as_str()) {
        return Ok((name, hex));
    }
    Err(BadArg::Color(name, known.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::color;

    #[test]
    fn chart_and_diagram_colors_pass_and_others_fail() {
        assert!(color("chartTileInk=#000").is_ok());
        assert!(color("public=#000").is_ok());
        assert!(color("nope=#000").is_err());
    }
}
