use super::super::out::Out;
use super::{page, repo_services};
use crate::closures::Closures;
use crate::facts::{DarwinHost, Facts, Host, NixosHost};
use crate::source::repo::Repo;
use crate::text::fill;
use crate::text::wiki::{hosts as t, NONE};
use crate::util::{human_count, human_size};
use anyhow::Result;
use std::path::Path;

fn join_or_dash(items: &[String]) -> String {
    if items.is_empty() {
        NONE.into()
    } else {
        items.join(", ")
    }
}

pub(super) fn page_hosts(
    out: &Out,
    src: &Path,
    facts: &Facts,
    repo: &Repo,
    closures: Option<&Closures>,
) -> Result<()> {
    let mut o = vec![t::TITLE.to_string()];
    for (host, f) in &facts.hosts {
        match f {
            Host::Nixos(n) => host_nixos(&mut o, host, n, repo, closures),
            Host::Darwin(d) => host_darwin(&mut o, host, d),
        }
    }
    page(out, &src.join("hosts.md"), &o)
}

fn host_nixos(
    o: &mut Vec<String>,
    host: &str,
    f: &NixosHost,
    repo: &Repo,
    closures: Option<&Closures>,
) {
    let svcs = repo_services(f, repo);
    let ports = |ps: &[u32]| join_or_dash(&ps.iter().map(u32::to_string).collect::<Vec<_>>());
    o.push(fill(t::NIXOS, &[("host", host)]));
    o.extend(f.description.clone());

    let platform = if f.platform.is_empty() {
        t::UNKNOWN_PLATFORM
    } else {
        &f.platform
    };
    let mut rows = vec![fill(t::PLATFORM, &[("platform", platform)])];
    if !f.state_version.is_empty() {
        rows.push(fill(t::STATE, &[("state", &f.state_version)]));
    }
    rows.push(fill(t::USERS, &[("users", &join_or_dash(&f.users))]));
    rows.push(fill(t::PACKAGES, &[("count", &f.pkg_count.to_string())]));
    if let Some(cs) = closures {
        let closure = match cs.hosts.get(host) {
            Some(c) => fill(
                t::CLOSURE_SIZE,
                &[
                    ("size", &human_size(c.total())),
                    ("paths", &human_count(c.len())),
                ],
            ),
            None => t::NOT_MEASURED.into(),
        };
        rows.push(fill(t::CLOSURE, &[("closure", &closure)]));
    }
    rows.push(fill(t::TCP, &[("ports", &ports(&f.tcp))]));
    rows.push(fill(t::UDP, &[("ports", &ports(&f.udp))]));
    rows.push(fill(
        t::SERVICES_COUNT,
        &[("count", &svcs.len().to_string())],
    ));
    o.push(fill(t::TABLE, &[("rows", &rows.join("\n"))]));

    if !svcs.is_empty() {
        let rows: Vec<String> = svcs
            .iter()
            .map(|(name, files)| {
                let files: Vec<String> = files
                    .iter()
                    .map(|f| fill(t::FILE, &[("file", f)]))
                    .collect();
                fill(t::SERVICE, &[("name", name), ("files", &files.join(" "))])
            })
            .collect();
        o.push(fill(t::SERVICES, &[("rows", &rows.join("\n"))]));
    }
}

fn host_darwin(o: &mut Vec<String>, host: &str, f: &DarwinHost) {
    o.push(fill(t::DARWIN, &[("host", host)]));
    o.push(
        f.description
            .clone()
            .unwrap_or_else(|| t::DARWIN_INTRO.into()),
    );
    for (title, items) in [
        (t::DAEMONS, &f.daemons),
        (t::AGENTS, &f.user_agents),
        (t::CASKS, &f.casks),
    ] {
        if !items.is_empty() {
            let mut sorted = items.clone();
            sorted.sort();
            o.push(fill(
                t::LIST,
                &[("title", title), ("items", &sorted.join(", "))],
            ));
        }
    }
}
