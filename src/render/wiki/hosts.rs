//! The Hosts page: one section per host, with its entry module's doc comment
//! as the description.

use super::super::out::{Out, MD_MARKER};
use super::repo_services;
use crate::closures::Closures;
use crate::facts::{DarwinHost, Facts, Host, NixosHost};
use crate::render::DocComments;
use crate::source::repo::Repo;
use crate::util::{human_count, human_size};
use anyhow::Result;
use std::path::Path;

fn fmt_ports(ports: &[u32]) -> String {
    if ports.is_empty() {
        "—".into()
    } else {
        ports
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn join_or_dash(items: &[String]) -> String {
    if items.is_empty() {
        "—".into()
    } else {
        items.join(", ")
    }
}

pub(super) fn page_hosts(
    out: &mut Out,
    src: &Path,
    facts: &Facts,
    repo: &Repo,
    docs: &DocComments,
    closures: Option<&Closures>,
) -> Result<()> {
    let mut o: Vec<String> = vec![MD_MARKER.into(), "".into(), "# Hosts".into(), "".into()];
    for (host, f) in &facts.hosts {
        match f {
            Host::Nixos(n) => host_nixos(&mut o, host, n, repo, docs.hosts.get(host), closures),
            Host::Darwin(d) => host_darwin(&mut o, host, d, docs.hosts.get(host)),
        }
    }
    out.write_auto(&src.join("hosts.md"), &o.join("\n"))
}

fn host_nixos(
    o: &mut Vec<String>,
    host: &str,
    f: &NixosHost,
    repo: &Repo,
    doc: Option<&String>,
    closures: Option<&Closures>,
) {
    let svcs = repo_services(f, repo);
    o.push(format!("## 🖥️ {host}"));
    o.push("".into());
    if let Some(doc) = doc {
        o.push(doc.clone());
        o.push("".into());
    }
    o.push("| | |".into());
    o.push("|---|---|".into());
    let platform = if f.platform.is_empty() {
        "?"
    } else {
        &f.platform
    };
    o.push(format!("| Platform | `{platform}` |"));
    if !f.state_version.is_empty() {
        o.push(format!("| State version | `{}` |", f.state_version));
    }
    o.push(format!("| Users | {} |", join_or_dash(&f.users)));
    o.push(format!("| System packages | {} |", f.pkg_count));
    // The row appears whenever closure measurement is on at all. A host the
    // `closures` list did not select says so, rather than silently omitting
    // the row, which would be indistinguishable from the feature being off.
    if let Some(cs) = closures {
        match cs.hosts.get(host) {
            Some(c) => o.push(format!(
                "| Closure | {} ({} paths) |",
                human_size(c.total()),
                human_count(c.len())
            )),
            None => o.push("| Closure | not measured |".into()),
        }
    }
    o.push(format!("| Open TCP ports | {} |", fmt_ports(&f.tcp)));
    o.push(format!("| Open UDP ports | {} |", fmt_ports(&f.udp)));
    o.push(format!("| Repo-configured services | {} |", svcs.len()));
    o.push("".into());
    if !svcs.is_empty() {
        o.push("**Services:**".into());
        o.push("".into());
        for (name, files) in &svcs {
            let files = files
                .iter()
                .map(|x| format!("`{x}`"))
                .collect::<Vec<_>>()
                .join(" ");
            o.push(format!("- **{name}** — {files}"));
        }
        o.push("".into());
    }
}

fn host_darwin(o: &mut Vec<String>, host: &str, f: &DarwinHost, doc: Option<&String>) {
    o.push(format!("## 🍏 {host}"));
    o.push("".into());
    if let Some(doc) = doc {
        o.push(doc.clone());
    } else {
        o.push("_nix-darwin host._".into());
    }
    o.push("".into());
    for (title, items) in [
        ("LaunchDaemons", &f.daemons),
        ("User agents", &f.user_agents),
        ("Homebrew casks", &f.casks),
    ] {
        if !items.is_empty() {
            let mut sorted = items.clone();
            sorted.sort();
            o.push(format!("**{title}:** {}", sorted.join(", ")));
            o.push("".into());
        }
    }
}
