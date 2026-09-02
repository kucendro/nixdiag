//! Mode A extraction: spawn `nix eval --json --apply <projection>` per host,
//! parallel across hosts. One merged projection eval per host.

use crate::facts::{Facts, Host, SCHEMA};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

const CORE_PROJ: &str = include_str!("../nix/projections/core.nix");

/// Optional `nixdiag = { … };` output declared in the documented flake:
/// defaults for mode A so `nixdiag gen` needs no flags. CLI flags override.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct FlakeConfig {
    pub out: Option<String>,
    pub title: Option<String>,
    pub extra_pages: BTreeMap<String, String>,
    pub extra_links: BTreeMap<String, String>,
    pub theme: Option<String>,
    pub background: Option<String>,
    pub colors: BTreeMap<String, String>,
    /// `@key` -> domain suffix for fqdn positions in annotations.
    pub domains: BTreeMap<String, String>,
    /// Annotation grammar edition these files are written against. Unset
    /// means "whatever the binary implements".
    pub grammar: Option<u32>,
    /// Warning categories promoted to errors, e.g. `[ "deprecated" ]`.
    pub deny: Vec<String>,
    /// Publish the `api/` tree. Unset means yes: the data is already computed
    /// and reads nothing new, the same reason the lock timeline ships on by
    /// default.
    pub api: Option<bool>,
    /// Revision the docs describe. A flake output may reference `self`, so a
    /// mode A consumer can write `nixdiag.revision = self.rev or null;` and
    /// nixdiag still never invokes git.
    pub revision: Option<String>,
    pub revision_time: Option<i64>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Nixos,
    Darwin,
}

pub struct HostRef {
    pub name: String,
    pub prefix: &'static str,
    pub kind: Kind,
}

fn nix_eval_json(
    flake: &Path,
    installable: &str,
    apply: Option<&str>,
    warn: bool,
) -> Option<serde_json::Value> {
    let mut cmd = Command::new("nix");
    cmd.args([
        "--extra-experimental-features",
        "nix-command flakes",
        "eval",
        "--json",
        installable,
    ]);
    if let Some(a) = apply {
        cmd.args(["--apply", a]);
    }
    let out = match cmd.current_dir(flake).output() {
        Ok(o) => o,
        Err(e) => {
            if warn {
                eprintln!("  ! eval {installable}: failed to spawn nix: {e}");
            }
            return None;
        }
    };
    if !out.status.success() {
        if warn {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let tail = stderr.trim().lines().last().unwrap_or("(no stderr)");
            eprintln!("  ! eval {installable}: {tail}");
        }
        return None;
    }
    serde_json::from_slice(&out.stdout).ok()
}

/// The declared `.#nixdiag` config, or defaults when the flake has none.
pub fn flake_config(flake: &Path) -> FlakeConfig {
    let Some(v) = nix_eval_json(flake, ".#nixdiag", None, false) else {
        return FlakeConfig::default();
    };
    match serde_json::from_value(v) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("  ! the flake's `nixdiag` output is not a valid config, ignoring: {e}");
            FlakeConfig::default()
        }
    }
}

pub fn discover(flake: &Path) -> Vec<HostRef> {
    let mut hosts = Vec::new();
    for (prefix, kind) in [
        ("nixosConfigurations", Kind::Nixos),
        ("darwinConfigurations", Kind::Darwin),
    ] {
        let names = nix_eval_json(
            flake,
            &format!(".#{prefix}"),
            Some("builtins.attrNames"),
            true,
        );
        if let Some(serde_json::Value::Array(names)) = names {
            for n in names {
                if let serde_json::Value::String(name) = n {
                    hosts.push(HostRef { name, prefix, kind });
                }
            }
        }
    }
    hosts
}

pub fn gather(flake: &Path, refs: &[HostRef]) -> Facts {
    let pairs: Vec<Option<(String, Host)>> = refs
        .par_iter()
        .map(|r| {
            let kind = match r.kind {
                Kind::Nixos => "nixos",
                Kind::Darwin => "darwin",
            };
            let proj = format!("host: ({CORE_PROJ}) {{ inherit host; kind = \"{kind}\"; }}");
            let installable = format!(".#{}.{}", r.prefix, r.name);
            let v = nix_eval_json(flake, &installable, Some(&proj), true)?;
            match serde_json::from_value::<Host>(v) {
                Ok(h) => Some((r.name.clone(), h)),
                Err(e) => {
                    eprintln!(
                        "  ! {}: projection output did not match facts model: {e}",
                        r.name
                    );
                    None
                }
            }
        })
        .collect();
    Facts {
        schema: SCHEMA,
        hosts: pairs.into_iter().flatten().collect(),
    }
}
