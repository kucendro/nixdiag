use super::{abs, RenderArgs};
use crate::closures::Closures;
use crate::render::{d2, RenderOpts, WikiOpts};
use crate::source::annotations;
use crate::text::cli::DEFAULT_TITLE;
use crate::text::{fill, messages as m};
use anyhow::{bail, Result};
use std::path::PathBuf;

fn pairs(specs: &[String], flag: &str, shape: &str) -> Result<Vec<(String, String)>> {
    specs
        .iter()
        .map(|s| match s.split_once('=') {
            Some((k, v)) if !k.is_empty() && !v.is_empty() => Ok((k.to_string(), v.to_string())),
            _ => bail!(fill(
                m::BAD_PAIR,
                &[("flag", flag), ("shape", shape), ("value", s)]
            )),
        })
        .collect()
}

pub(super) fn to_render_opts(r: &RenderArgs, closures: Option<Closures>) -> Result<RenderOpts> {
    let extra_pages = pairs(&r.extra_pages, "--extra-page", "TITLE=FILE")?
        .into_iter()
        .map(|(t, p)| (t, PathBuf::from(p)))
        .collect();
    Ok(RenderOpts {
        repo: abs(&r.repo),
        out: abs(&r.out),
        wiki: WikiOpts {
            title: r.title.clone().unwrap_or_else(|| DEFAULT_TITLE.into()),
            extra_pages,
            extra_links: pairs(&r.extra_links, "--extra-link", "TITLE=NAME.md")?,
        },
        svg: !r.no_svg,
        style: to_style(r)?,
        domains: pairs(&r.domains, "--domain", "KEY=DOMAIN")?
            .into_iter()
            .collect(),
        grammar: annotations::resolve_edition(r.grammar)?,
        deny: r.deny.clone(),
        closures,
    })
}

fn to_style(r: &RenderArgs) -> Result<d2::D2Style> {
    let colors = pairs(&r.colors, "--color", "NAME=#HEX")?;
    for (n, _) in &colors {
        if !d2::PALETTE.iter().any(|(p, ..)| p == n) {
            let known: Vec<&str> = d2::PALETTE.iter().map(|(p, ..)| *p).collect();
            bail!(fill(
                m::UNKNOWN_COLOR,
                &[("name", n), ("palette", &known.join(", "))]
            ));
        }
    }
    Ok(d2::D2Style {
        dark: r.theme.as_deref() != Some("light"),
        background: Some(r.background.clone().unwrap_or_else(|| "transparent".into())),
        colors,
    })
}
