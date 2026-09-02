//! The published API tree: `api/v1/*.json`, the OpenAPI document that
//! describes them, and the Scalar reference shim.
//!
//! Façade in the shape of `wiki/mod.rs` — submodules are private and each
//! owns one document. They *build* payloads and never touch `Out`, so the
//! writing (and the choice of `WKind`) stays in one place here and every
//! builder is unit-testable without a filesystem.

mod closures;
mod hosts;
mod inputs;
mod openapi;
mod scalar;
mod services;
mod snapshot;
mod topology;

use super::out::{Out, WKind};
use crate::api::{self, Meta, API_VERSION};
use crate::closures::Closures;
use crate::facts::Facts;
use crate::source::annotations::Model;
use crate::source::flakelock::Lock;
use crate::source::repo::Repo;
use anyhow::Result;
use std::path::PathBuf;

/// Everything the documents are built from — the same inputs the wiki pages
/// draw on, minus the doc comments, which are prose and belong in the book.
pub struct ApiData<'a> {
    pub facts: &'a Facts,
    pub repo: &'a Repo,
    pub model: &'a Model,
    pub lock: Option<&'a Lock>,
    pub closures: Option<&'a Closures>,
}

pub struct ApiOpts {
    /// Resolved annotation edition, reported in every document's `meta`.
    pub grammar: u32,
    /// Supplied by the caller — `render` shells out to no git and reads no
    /// clock. `None` in mode A unless CI passes one.
    pub revision: Option<api::Revision>,
    /// Emit the vendored Scalar reference page. Mode B only: the CLI has no
    /// bundle to point at and must not fetch one.
    pub scalar: bool,
}

/// Every endpoint, in the order the service document lists them.
fn links(has_lock: bool, has_closures: bool) -> Vec<api::Link> {
    let mut v = vec![
        ("hosts.json", "Hosts, platforms, open ports and users"),
        ("services.json", "Services this repo configures and where"),
        ("topology.json", "Annotated nodes, edges and endpoints"),
    ];
    if has_lock {
        v.push(("inputs.json", "Flake inputs, lock dates and duplicates"));
    }
    if has_closures {
        v.push(("closures.json", "Per-host closure sizes by package"));
    }
    v.push(("snapshot.json", "Totals plus revision identity, for trends"));
    v.push(("openapi.json", "This API, as an OpenAPI 3.1 document"));
    v.into_iter()
        .map(|(f, description)| api::Link {
            path: format!("/api/{API_VERSION}/{f}"),
            description,
        })
        .collect()
}

pub fn generate(out: &mut Out, opts: &ApiOpts, d: &ApiData) -> Result<()> {
    let root = PathBuf::from("api");
    let v = root.join(API_VERSION);
    let meta = || Meta::new(opts.grammar);

    out.write_json(
        &v.join("index.json"),
        &api::Index {
            meta: meta(),
            endpoints: links(d.lock.is_some(), d.closures.is_some()),
        },
        WKind::Auto,
    )?;
    out.write_json(
        &v.join("hosts.json"),
        &hosts::build(meta(), d.facts, d.repo, d.model),
        WKind::Auto,
    )?;
    out.write_json(
        &v.join("services.json"),
        &services::build(meta(), d.facts, d.repo),
        WKind::Auto,
    )?;
    out.write_json(
        &v.join("topology.json"),
        &topology::build(meta(), d.facts, d.model),
        WKind::Auto,
    )?;
    // A flake without a lock is legitimate; the endpoint is simply absent,
    // which is why `index.json` lists it conditionally.
    if let Some(lock) = d.lock {
        out.write_json(
            &v.join("inputs.json"),
            &inputs::build(meta(), lock),
            WKind::Auto,
        )?;
    }
    if let Some(c) = d.closures {
        out.write_json(
            &v.join("closures.json"),
            &closures::build(meta(), d.facts, c),
            WKind::Auto,
        )?;
    }
    // Volatile: it carries the revision, so gating it would make every
    // consumer's `check` red on each commit.
    out.write_json(
        &v.join("snapshot.json"),
        &snapshot::build(meta(), opts.revision.clone(), d),
        WKind::Volatile,
    )?;
    out.write_json(
        &v.join("openapi.json"),
        &openapi::build(d.lock.is_some(), d.closures.is_some()),
        WKind::Auto,
    )?;
    if opts.scalar {
        out.write_auto(&root.join("index.html"), &scalar::page())?;
    }
    Ok(())
}
