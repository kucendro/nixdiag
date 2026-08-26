mod annotations;
mod d2;
mod doccomment;
mod eval;
mod facts;
mod modules;
mod output;
mod render;
mod repo;
mod topology;
mod util;
mod wiki;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use facts::Facts;
use output::WKind;
use render::{render_all, RenderOpts};
use std::path::{Path, PathBuf};
use wiki::WikiOpts;

#[derive(Parser)]
#[command(
    name = "nixdiag",
    version,
    about = "Static infrastructure docs from any Nix flake"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Args)]
struct FlakeArgs {
    /// Flake directory to evaluate
    #[arg(long, default_value = ".")]
    flake: PathBuf,
    /// Restrict to these hosts (default: all discovered)
    hosts: Vec<String>,
}

#[derive(Args)]
struct RenderArgs {
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
        #[command(flatten)]
        render: RenderArgs,
    },
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Facts(f) => cmd_facts(f),
        Cmd::Render {
            facts,
            repo,
            out,
            render,
        } => cmd_render(facts, repo, out, render),
        Cmd::Gen { flake, out, render } => cmd_gen(flake, out, render).map(|_| ()),
        Cmd::Check { flake, out, render } => cmd_check(flake, out, render),
    }
}

fn abs(p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(p)
    }
}

fn gather_facts(args: &FlakeArgs) -> Result<(PathBuf, Facts)> {
    let flake = abs(&args.flake);
    if !flake.join("flake.nix").exists() {
        bail!("{} has no flake.nix", flake.display());
    }
    let all = eval::discover(&flake);
    let refs = if args.hosts.is_empty() {
        all
    } else {
        let known: Vec<&str> = all.iter().map(|h| h.name.as_str()).collect();
        let unknown: Vec<&String> = args
            .hosts
            .iter()
            .filter(|h| !known.contains(&h.as_str()))
            .collect();
        if !unknown.is_empty() {
            bail!("unknown host(s): {unknown:?}; known: {known:?}");
        }
        args.hosts
            .iter()
            .map(|h| all.iter().position(|r| &r.name == h).unwrap())
            .collect::<Vec<_>>()
            .into_iter()
            .map(|i| {
                let r = &all[i];
                eval::HostRef {
                    name: r.name.clone(),
                    prefix: r.prefix,
                    kind: r.kind,
                }
            })
            .collect()
    };
    Ok((flake.clone(), eval::gather(&flake, &refs)))
}

fn parse_extra_pages(specs: &[String]) -> Result<Vec<(String, PathBuf)>> {
    specs
        .iter()
        .map(|s| match s.split_once('=') {
            Some((t, p)) if !t.is_empty() && !p.is_empty() => Ok((t.to_string(), PathBuf::from(p))),
            _ => bail!("--extra-page expects TITLE=FILE, got: {s}"),
        })
        .collect()
}

/// Declared flake config first, CLI flags on top.
fn to_render_opts(
    repo: PathBuf,
    out: PathBuf,
    r: &RenderArgs,
    cfg: &eval::FlakeConfig,
) -> Result<RenderOpts> {
    let mut extra_pages: Vec<(String, PathBuf)> = cfg
        .extra_pages
        .iter()
        .map(|(t, p)| (t.clone(), PathBuf::from(p)))
        .collect();
    extra_pages.extend(parse_extra_pages(&r.extra_pages)?);
    let mut extra_links: Vec<(String, String)> = cfg
        .extra_links
        .iter()
        .map(|(t, n)| (t.clone(), n.clone()))
        .collect();
    extra_links.extend(
        parse_extra_pages(&r.extra_links)?
            .into_iter()
            .map(|(t, p)| (t, p.to_string_lossy().into_owned())),
    );
    Ok(RenderOpts {
        repo,
        out,
        wiki: WikiOpts {
            title: r
                .title
                .clone()
                .or_else(|| cfg.title.clone())
                .unwrap_or_else(|| "Infrastructure wiki".into()),
            extra_pages,
            extra_links,
        },
        svg: !r.no_svg,
    })
}

/// --out flag, then the flake's declared `nixdiag.out` (relative to the
/// flake), then <flake>/docs.
fn resolve_out(cli: Option<PathBuf>, cfg: &eval::FlakeConfig, flake: &Path) -> PathBuf {
    cli.map(|o| abs(&o))
        .or_else(|| cfg.out.as_ref().map(|o| flake.join(o)))
        .unwrap_or_else(|| flake.join("docs"))
}

fn cmd_facts(args: FlakeArgs) -> Result<()> {
    let (_, facts) = gather_facts(&args)?;
    println!("{}", serde_json::to_string_pretty(&facts)?);
    Ok(())
}

fn cmd_render(facts_path: PathBuf, repo: PathBuf, out: PathBuf, r: RenderArgs) -> Result<()> {
    let text = if facts_path == Path::new("-") {
        std::io::read_to_string(std::io::stdin())?
    } else {
        std::fs::read_to_string(&facts_path)
            .with_context(|| format!("reading {}", facts_path.display()))?
    };
    let mut facts: Facts = serde_json::from_str(&text).context("parsing facts.json")?;
    let cfg = eval::FlakeConfig::default();
    render_all(
        &mut facts,
        &to_render_opts(abs(&repo), abs(&out), &r, &cfg)?,
    )?;
    Ok(())
}

fn cmd_gen(args: FlakeArgs, out: Option<PathBuf>, r: RenderArgs) -> Result<output::Out> {
    let (flake, mut facts) = gather_facts(&args)?;
    let cfg = eval::flake_config(&flake);
    let out = resolve_out(out, &cfg, &flake);
    render_all(&mut facts, &to_render_opts(flake, out, &r, &cfg)?)
}

fn cmd_check(args: FlakeArgs, out: Option<PathBuf>, mut r: RenderArgs) -> Result<()> {
    let flake = abs(&args.flake);
    let cfg = eval::flake_config(&flake);
    let committed = resolve_out(out, &cfg, &flake);
    let tmp = std::env::temp_dir().join(format!("nixdiag-check-{}", std::process::id()));
    if tmp.exists() {
        std::fs::remove_dir_all(&tmp)?;
    }
    r.no_svg = true; // SVG output varies with the d2 version; compare sources only
    let (_, mut facts) = gather_facts(&args)?;
    let rendered = render_all(&mut facts, &to_render_opts(flake, tmp.clone(), &r, &cfg)?)?;

    let mut drift: Vec<PathBuf> = Vec::new();
    for w in &rendered.manifest {
        if w.kind != WKind::Auto {
            continue;
        }
        let fresh = std::fs::read(tmp.join(&w.rel)).unwrap_or_default();
        let existing = std::fs::read(committed.join(&w.rel)).unwrap_or_default();
        if fresh != existing {
            drift.push(w.rel.clone());
        }
    }
    std::fs::remove_dir_all(&tmp).ok();

    if drift.is_empty() {
        println!("docs are up to date");
        Ok(())
    } else {
        for d in &drift {
            eprintln!("out of date: {}", committed.join(d).display());
        }
        bail!("docs drifted from the config — run: nixdiag gen");
    }
}
