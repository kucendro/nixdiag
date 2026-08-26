//! mdBook wiki source — port of gen-wiki.py.

use crate::annotations::Model;
use crate::facts::{DarwinHost, Facts, Host, NixosHost};
use crate::output::{Out, MD_MARKER};
use crate::render::DocComments;
use crate::repo::Repo;
use anyhow::{bail, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub struct WikiOpts {
    pub title: String,
    /// (link title, source path) — appended to SUMMARY and copied into src/.
    pub extra_pages: Vec<(String, PathBuf)>,
    /// (link title, file name) — SUMMARY entry only, for pages some other
    /// tool writes into wiki/src itself.
    pub extra_links: Vec<(String, String)>,
}

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

pub fn generate(
    facts: &Facts,
    repo: &Repo,
    out: &mut Out,
    opts: &WikiOpts,
    docs: &DocComments,
    model: &Model,
) -> Result<()> {
    let wiki = PathBuf::from("wiki");
    let src = wiki.join("src");

    book_toml(out, &wiki, &opts.title)?;
    let mut extra = copy_extra_pages(out, &src, &opts.extra_pages)?;
    extra.extend(opts.extra_links.iter().cloned());
    page_summary(out, &src, &extra)?;
    page_index(out, &src)?;
    page_architecture(out, &src)?;
    page_hosts(out, &src, facts, repo, docs)?;
    page_services(out, &src, facts, repo, docs)?;
    page_endpoints(out, &src, facts, model)?;
    Ok(())
}

fn book_toml(out: &mut Out, wiki: &Path, title: &str) -> Result<()> {
    out.write_once(
        &wiki.join("book.toml"),
        &format!(
            "[book]\n\
             title = \"{title}\"\n\
             src = \"src\"\n\n\
             [output.html]\n\
             default-theme = \"navy\"\n\
             preferred-dark-theme = \"navy\"\n\
             no-section-label = true\n"
        ),
    )
}

fn copy_extra_pages(
    out: &mut Out,
    src: &Path,
    pages: &[(String, PathBuf)],
) -> Result<Vec<(String, String)>> {
    let mut links = Vec::new();
    for (title, source) in pages {
        let fname = source
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();
        if fname.is_empty() {
            bail!("--extra-page {title}: source has no file name");
        }
        let dest_rel = src.join(&fname);
        let dest = out.root.join(&dest_rel);
        if !source.exists() {
            bail!("--extra-page {title}: {} not found", source.display());
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(source, &dest)?;
        println!("wrote {}", dest.display());
        out.record_extra(&dest_rel);
        links.push((title.clone(), fname));
    }
    Ok(links)
}

fn page_summary(out: &mut Out, src: &Path, extra: &[(String, String)]) -> Result<()> {
    let mut text = format!(
        "{MD_MARKER}\n\n\
         # Summary\n\n\
         - [Overview](./index.md)\n\
         - [Architecture](./architecture.md)\n\
         - [Hosts](./hosts.md)\n\
         - [Services](./services.md)\n\
         - [Endpoints](./endpoints.md)\n"
    );
    for (title, fname) in extra {
        text.push_str(&format!("- [{title}](./{fname})\n"));
    }
    out.write_auto(&src.join("SUMMARY.md"), &text)
}

fn page_index(out: &mut Out, src: &Path) -> Result<()> {
    out.write_once(
        &src.join("index.md"),
        "# Infrastructure wiki\n\n\
         _Hand-written overview goes here_ — the big picture, where a newcomer \
         should start, and *why* things are the way they are.\n\n\
         Everything else in this wiki (Architecture, Hosts, Services, Endpoints) \
         is **auto-generated from the Nix configuration**, so it is always \
         current. This page is the one you edit by hand.\n",
    )
}

fn page_architecture(out: &mut Out, src: &Path) -> Result<()> {
    for svg in ["topology.svg", "modules.svg"] {
        let from = out.root.join(svg);
        if from.exists() {
            let rel = src.join(svg);
            std::fs::create_dir_all(out.root.join(src))?;
            std::fs::copy(&from, out.root.join(&rel))?;
            out.record_svg(&rel);
        }
    }
    out.write_auto(
        &src.join("architecture.md"),
        &format!(
            "{MD_MARKER}\n\n\
             # Architecture\n\n\
             ## Data-flow topology\n\n\
             What talks to what across the fleet.\n\n\
             ![Data-flow topology](./topology.svg)\n\n\
             ## Module tree\n\n\
             How each host is assembled from the module files in this repo.\n\n\
             ![Module tree](./modules.svg)\n"
        ),
    )
}

/// Services this repo actually configures: name -> repo-relative files.
fn repo_services(n: &NixosHost, repo: &Repo) -> BTreeMap<String, Vec<String>> {
    let mut svcs = BTreeMap::new();
    for item in &n.services {
        let files = repo.repo_files(&item.files);
        if !files.is_empty() {
            svcs.insert(item.name.clone(), files);
        }
    }
    svcs
}

fn page_hosts(
    out: &mut Out,
    src: &Path,
    facts: &Facts,
    repo: &Repo,
    docs: &DocComments,
) -> Result<()> {
    let mut o: Vec<String> = vec![MD_MARKER.into(), "".into(), "# Hosts".into(), "".into()];
    for (host, f) in &facts.hosts {
        match f {
            Host::Nixos(n) => host_nixos(&mut o, host, n, repo, docs.hosts.get(host)),
            Host::Darwin(d) => host_darwin(&mut o, host, d, docs.hosts.get(host)),
        }
    }
    out.write_auto(&src.join("hosts.md"), &o.join("\n"))
}

fn host_nixos(o: &mut Vec<String>, host: &str, f: &NixosHost, repo: &Repo, doc: Option<&String>) {
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
    o.push(format!("| Open TCP ports | {} |", fmt_ports(&f.tcp)));
    o.push(format!("| Open UDP ports | {} |", fmt_ports(&f.udp)));
    o.push(format!("| Repo-configured services | {} |", svcs.len()));
    o.push("".into());
    if !svcs.is_empty() {
        o.push("**Services** (configured in this repo):".into());
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

fn page_services(
    out: &mut Out,
    src: &Path,
    facts: &Facts,
    repo: &Repo,
    docs: &DocComments,
) -> Result<()> {
    // name -> (hosts, files)
    let mut index: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
    for (host, f) in &facts.hosts {
        let Some(n) = f.as_nixos() else { continue };
        for (name, files) in repo_services(n, repo) {
            let e = index.entry(name).or_default();
            e.0.insert(host.clone());
            e.1.extend(files);
        }
    }
    let mut o: Vec<String> = vec![
        MD_MARKER.into(),
        "".into(),
        "# Services".into(),
        "".into(),
        "Every service this repo configures, the host(s) that run it, and the \
         file that defines it."
            .into(),
        "".into(),
        "| Service | Hosts | Defined in |".into(),
        "|---|---|---|".into(),
    ];
    for (name, (hosts, files)) in &index {
        let hosts = hosts.iter().cloned().collect::<Vec<_>>().join(", ");
        let files = files
            .iter()
            .map(|x| format!("`{x}`"))
            .collect::<Vec<_>>()
            .join(" ");
        o.push(format!("| **{name}** | {hosts} | {files} |"));
    }
    if index.is_empty() {
        o.push("| — | — | — |".into());
    }
    // Doc-commented services get a section below the table.
    for (name, (_, files)) in &index {
        let Some(doc) = files.iter().find_map(|f| docs.files.get(f)) else {
            continue;
        };
        o.push("".into());
        o.push(format!("## {name}"));
        o.push("".into());
        o.push(doc.clone());
    }
    out.write_auto(&src.join("services.md"), &o.join("\n"))
}

fn page_endpoints(out: &mut Out, src: &Path, facts: &Facts, model: &Model) -> Result<()> {
    let mut o: Vec<String> = vec![
        MD_MARKER.into(),
        "".into(),
        "# Endpoints".into(),
        "".into(),
        "Declared service endpoints across the fleet, from `#: expose` \
         annotations in the module files."
            .into(),
        "".into(),
        "| Endpoint | Port | Scope | Host | Service |".into(),
        "|---|---|---|---|---|".into(),
    ];
    // (endpoint, port, scope, host, service)
    let mut rows: Vec<(String, String, String, String, String)> = Vec::new();
    let mut push = |host: &str, unit: Option<&str>, info: &crate::annotations::NodeInfo| {
        for e in &info.exposes {
            let endpoint = e
                .name
                .clone()
                .unwrap_or_else(|| format!("{host}:{}", e.port));
            let port = format!("{}{}", e.port, if e.udp { "/udp" } else { "" });
            let scope = model
                .effective_scope(host, unit, e)
                .map(|s| s.label().to_string())
                .unwrap_or_else(|| "—".into());
            rows.push((
                endpoint,
                port,
                scope,
                host.to_string(),
                unit.unwrap_or("—").to_string(),
            ));
        }
    };
    for host in facts.hosts.keys() {
        if let Some(info) = model.hosts.get(host) {
            push(host, None, info);
        }
        for ((h, unit), info) in &model.units {
            if h == host {
                push(host, Some(unit), info);
            }
        }
    }
    rows.sort();
    for (endpoint, port, scope, host, service) in &rows {
        o.push(format!(
            "| `{endpoint}` | {port} | {scope} | {host} | {service} |"
        ));
    }
    if rows.is_empty() {
        o.push("| — | — | — | — | — |".into());
    }
    out.write_auto(&src.join("endpoints.md"), &o.join("\n"))
}
