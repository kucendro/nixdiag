use super::options::{abs, resolve_out, to_render_opts};
use super::{FlakeArgs, RenderArgs};
use crate::closures::Closures;
use crate::eval;
use crate::facts::Facts;
use crate::render::{render_all, Out, WKind};
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

fn closures_need_mode_b() -> anyhow::Error {
    anyhow::anyhow!(
        "closure metrics require every host's system to be built, which this \
         command cannot do purely.\nDeclare them in your flake instead:\n\n    \
         nixdiag.lib.mkDocs {{ closures = true; /* … */ }}\n"
    )
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

pub(super) fn cmd_facts(args: FlakeArgs) -> Result<()> {
    let (_, facts) = gather_facts(&args)?;
    println!("{}", serde_json::to_string_pretty(&facts)?);
    Ok(())
}

pub(super) fn cmd_render(
    facts_path: PathBuf,
    repo: PathBuf,
    out: PathBuf,
    closures_path: Option<PathBuf>,
    r: RenderArgs,
) -> Result<()> {
    let text = if facts_path == Path::new("-") {
        std::io::read_to_string(std::io::stdin())?
    } else {
        std::fs::read_to_string(&facts_path)
            .with_context(|| format!("reading {}", facts_path.display()))?
    };
    let mut facts: Facts = serde_json::from_str(&text).context("parsing facts.json")?;
    let closures = closures_path
        .map(|p| -> Result<Closures> {
            let text =
                std::fs::read_to_string(&p).with_context(|| format!("reading {}", p.display()))?;
            serde_json::from_str(&text).context("parsing closures.json")
        })
        .transpose()?;
    let cfg = eval::FlakeConfig::default();
    render_all(
        &mut facts,
        &to_render_opts(abs(&repo), abs(&out), &r, &cfg, closures)?,
    )?;
    Ok(())
}

pub(super) fn cmd_gen(
    args: FlakeArgs,
    out: Option<PathBuf>,
    closures: bool,
    r: RenderArgs,
) -> Result<Out> {
    if closures {
        return Err(closures_need_mode_b());
    }
    let (flake, mut facts) = gather_facts(&args)?;
    let cfg = eval::flake_config(&flake);
    let out = resolve_out(out, &cfg, &flake);
    render_all(&mut facts, &to_render_opts(flake, out, &r, &cfg, None)?)
}

pub(super) fn cmd_check(
    args: FlakeArgs,
    out: Option<PathBuf>,
    closures: bool,
    mut r: RenderArgs,
) -> Result<()> {
    if closures {
        return Err(closures_need_mode_b());
    }
    let flake = abs(&args.flake);
    let cfg = eval::flake_config(&flake);
    let committed = resolve_out(out, &cfg, &flake);
    let tmp = std::env::temp_dir().join(format!("nixdiag-check-{}", std::process::id()));
    if tmp.exists() {
        std::fs::remove_dir_all(&tmp)?;
    }
    r.no_svg = true; // SVG output varies with the d2 version; compare sources only
    let (_, mut facts) = gather_facts(&args)?;
    let rendered = render_all(
        &mut facts,
        &to_render_opts(flake, tmp.clone(), &r, &cfg, None)?,
    )?;

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
        bail!(
            "docs drifted from the config — run: nixdiag gen\n\
             (if you just upgraded nixdiag, a cosmetic output change is expected; \
             see the CHANGELOG)"
        );
    }
}
