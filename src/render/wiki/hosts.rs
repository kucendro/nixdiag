use super::{code, codes, repo_services, size_paths, table, Wiki};
use crate::closures::Closures;
use crate::conf::files::page;
use crate::facts::{DarwinHost, Host, NixosHost};
use crate::source::repo::Repo;
use crate::text::wiki::{hosts as t, KV, NONE, NOT_MEASURED};
use anyhow::Result;

fn join_or_dash(items: &[String]) -> String {
    if items.is_empty() {
        NONE.into()
    } else {
        items.join(", ")
    }
}

pub(super) fn page_hosts(w: &Wiki) -> Result<()> {
    let mut o = vec![t::TITLE.to_string()];
    for (host, f) in &w.facts.hosts {
        match f {
            Host::Nixos(n) => host_nixos(&mut o, host, n, w.repo, w.closures),
            Host::Darwin(d) => host_darwin(&mut o, host, d),
        }
    }
    w.page(page::HOSTS, &o)
}

fn host_nixos(
    o: &mut Vec<String>,
    host: &str,
    f: &NixosHost,
    repo: &Repo,
    closures: Option<&Closures>,
) {
    let svcs = repo_services(&f.base, repo);
    let ports = |ps: &[u32]| join_or_dash(&ps.iter().map(u32::to_string).collect::<Vec<_>>());
    o.push(t::nixos(host));
    o.extend(f.base.description.clone());

    let platform = if f.platform.is_empty() {
        t::UNKNOWN_PLATFORM
    } else {
        &f.platform
    };
    let mut rows = vec![[t::PLATFORM.into(), code(platform)]];
    if !f.state_version.is_empty() {
        rows.push([t::STATE.into(), code(&f.state_version)]);
    }
    rows.push([t::USERS.into(), join_or_dash(&f.users)]);
    rows.push([t::PACKAGES.into(), f.pkg_count.to_string()]);
    if let Some(cs) = closures {
        let closure = cs.hosts.get(host).map(|c| size_paths(&c.total()));
        rows.push([t::CLOSURE.into(), closure.unwrap_or(NOT_MEASURED.into())]);
    }
    rows.push([t::TCP.into(), ports(&f.tcp)]);
    rows.push([t::UDP.into(), ports(&f.udp)]);
    rows.push([t::SERVICES_COUNT.into(), svcs.len().to_string()]);
    o.push(table(KV, rows));

    if !svcs.is_empty() {
        let rows: Vec<String> = svcs
            .iter()
            .map(|(name, files)| t::service(name, &codes(files, " ")))
            .collect();
        o.push(t::services(&rows.join("\n")));
    }
}

fn host_darwin(o: &mut Vec<String>, host: &str, f: &DarwinHost) {
    o.push(t::darwin(host));
    o.push(
        f.base
            .description
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
            o.push(t::list(title, &sorted.join(", ")));
        }
    }
}
