//! Settings resolution. Every knob has the same override order: the flake's
//! declared `nixdiag` output first, CLI flags on top.

use super::RenderArgs;
use crate::api;
use crate::closures::Closures;
use crate::eval;
use crate::render::{d2, ApiOpts, RenderOpts, WikiOpts};
use crate::source::annotations;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub(super) fn abs(p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(p)
    }
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
pub(super) fn to_render_opts(
    repo: PathBuf,
    out: PathBuf,
    r: &RenderArgs,
    cfg: &eval::FlakeConfig,
    closures: Option<Closures>,
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
    let mut domains = cfg.domains.clone();
    for s in &r.domains {
        match s.split_once('=') {
            Some((k, v)) if !k.is_empty() && !v.is_empty() => {
                domains.insert(k.to_string(), v.to_string());
            }
            _ => bail!("--domain expects KEY=DOMAIN, got: {s}"),
        }
    }
    let mut deny = cfg.deny.clone();
    deny.extend(r.deny.iter().cloned());
    let grammar = annotations::resolve_edition(r.grammar.or(cfg.grammar))?;
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
        style: to_style(r, cfg)?,
        domains,
        grammar,
        deny,
        closures,
        api: to_api_opts(r, cfg, grammar),
    })
}

/// The `api/` tree is on unless something turns it off; a flag beats the
/// flake, like every other setting.
fn to_api_opts(r: &RenderArgs, cfg: &eval::FlakeConfig, grammar: u32) -> Option<ApiOpts> {
    if r.no_api || cfg.api == Some(false) {
        return None;
    }
    let revision = r
        .revision
        .clone()
        .or_else(|| cfg.revision.clone())
        .map(|id| api::Revision {
            // `self.dirtyRev` spells an unclean tree `<rev>-dirty`, so the
            // flag is already in the identifier; a dashboard can drop those
            // points from a trend without a second field to thread through.
            dirty: id.ends_with("-dirty"),
            id,
            time: r.revision_time.or(cfg.revision_time),
        });
    Some(ApiOpts {
        grammar,
        revision,
        scalar: r.scalar,
    })
}

fn to_style(r: &RenderArgs, cfg: &eval::FlakeConfig) -> Result<d2::D2Style> {
    let dark = match r.theme.as_deref().or(cfg.theme.as_deref()) {
        None | Some("dark") => true,
        Some("light") => false,
        Some(t) => bail!("theme must be light or dark, got: {t}"),
    };
    let mut colors: Vec<(String, String)> = cfg
        .colors
        .iter()
        .map(|(n, v)| (n.clone(), v.clone()))
        .collect();
    for s in &r.colors {
        match s.split_once('=') {
            Some((n, v)) if !n.is_empty() && !v.is_empty() => {
                colors.push((n.to_string(), v.to_string()))
            }
            _ => bail!("--color expects NAME=#HEX, got: {s}"),
        }
    }
    for (n, _) in &colors {
        if !d2::PALETTE.iter().any(|(p, ..)| p == n) {
            let known: Vec<&str> = d2::PALETTE.iter().map(|(p, ..)| *p).collect();
            bail!("unknown color {n}; palette: {}", known.join(", "));
        }
    }
    Ok(d2::D2Style {
        dark,
        background: r
            .background
            .clone()
            .or_else(|| cfg.background.clone())
            .or_else(|| Some("transparent".into())),
        colors,
    })
}

/// --out flag, then the flake's declared `nixdiag.out` (relative to the
/// flake), then <flake>/docs.
pub(super) fn resolve_out(cli: Option<PathBuf>, cfg: &eval::FlakeConfig, flake: &Path) -> PathBuf {
    cli.map(|o| abs(&o))
        .or_else(|| cfg.out.as_ref().map(|o| flake.join(o)))
        .unwrap_or_else(|| flake.join("docs"))
}
