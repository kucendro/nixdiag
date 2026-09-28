use super::{code, codes, diagram, repo_services, size_paths, table, Page, Wiki};
use crate::closures::Closures;
use crate::conf::files::{diagram::modules, page};
use crate::facts::{DarwinHost, Host, NixosHost};
use crate::source::repo::Repo;
use crate::text::wiki::{hosts as t, KV, NONE, NOT_MEASURED};
use anyhow::Result;
use itertools::Itertools;
use std::fmt::Display;

fn join_or_dash(items: &[impl Display]) -> String {
    if items.is_empty() {
        NONE.into()
    } else {
        items.iter().join(", ")
    }
}

pub(super) struct Hosts;

impl Page for Hosts {
    fn file(&self) -> &'static str {
        page::HOSTS
    }

    fn title(&self) -> &'static str {
        t::TITLE
    }

    fn body(&self, w: &Wiki) -> Result<Option<Vec<String>>> {
        let mut o = Vec::new();
        for (host, f) in &w.facts.hosts {
            match f {
                Host::Nixos(n) => host_nixos(&mut o, host, n, w.repo, w.closures),
                Host::Darwin(d) => host_darwin(&mut o, host, d),
            }
            if let Some(board) = diagram(w, &modules(host)) {
                o.extend([t::MODULES.into(), board]);
            }
        }
        Ok(Some(o))
    }
}

fn host_nixos(
    o: &mut Vec<String>,
    host: &str,
    f: &NixosHost,
    repo: &Repo,
    closures: Option<&Closures>,
) {
    let svcs = repo_services(&f.base, repo);
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
    rows.push([t::TCP.into(), join_or_dash(&f.tcp)]);
    rows.push([t::UDP.into(), join_or_dash(&f.udp)]);
    rows.push([t::SERVICES_COUNT.into(), svcs.len().to_string()]);
    o.push(table(KV, rows));

    if !svcs.is_empty() {
        let mut rows = svcs
            .iter()
            .map(|(name, files)| t::service(name, &codes(files, " ")));
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
            o.push(t::list(title, &items.iter().sorted().join(", ")));
        }
    }
}
