use super::d2::{preamble, write_and_render, D2Style};
use super::out::Out;
use crate::facts::{Facts, Host};
use crate::source::imports::{build_import_graph, host_entry_modules, rel_str};
use crate::source::repo::{rel_from_store, Repo};
use crate::text::d2::modules as t;
use crate::text::{fill, messages as m};
use crate::util::sanitize;
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};

fn d2_path(rel: &str) -> String {
    rel.split('/').map(sanitize).collect::<Vec<_>>().join(".")
}

#[derive(Default)]
struct FileMeta {
    svcs: Vec<String>,
    progs: Vec<String>,
}

#[derive(Default)]
struct Tree {
    dirs: BTreeMap<String, Tree>,
    files: BTreeMap<String, FileMeta>,
}

impl Tree {
    fn add_file(&mut self, rel: &str) -> &mut FileMeta {
        let parts: Vec<&str> = rel.split('/').collect();
        let mut node = self;
        for d in &parts[..parts.len() - 1] {
            node = node.dirs.entry(d.to_string()).or_default();
        }
        node.files
            .entry(parts[parts.len() - 1].to_string())
            .or_default()
    }

    fn emit(&self, out: &mut Vec<String>, indent: usize) {
        let pad = "  ".repeat(indent);
        let line = |template: &str, name: &str| {
            fill(
                template,
                &[("pad", &pad), ("id", &sanitize(name)), ("name", name)],
            )
        };
        for (name, sub) in &self.dirs {
            out.push(line(t::DIR_OPEN, name));
            sub.emit(out, indent + 1);
            out.push(line(t::CLOSE, ""));
        }
        for (fname, meta) in &self.files {
            if meta.svcs.is_empty() && meta.progs.is_empty() {
                out.push(line(t::FILE, fname));
                continue;
            }
            out.push(line(t::FILE_OPEN, fname));
            let mut svcs = meta.svcs.clone();
            svcs.sort();
            for s in svcs {
                out.push(line(t::SERVICE, &s));
            }
            let mut progs = meta.progs.clone();
            progs.sort();
            for p in progs {
                out.push(line(t::PROGRAM, &p));
            }
            out.push(line(t::CLOSE, ""));
        }
    }
}

pub fn generate(
    facts: &Facts,
    repo: &Repo,
    out: &Out,
    render_svg: bool,
    style: &D2Style,
) -> Result<()> {
    let mut tree = Tree::default();
    let mut host_edges: Vec<(String, String)> = Vec::new();
    let mut import_edges: BTreeSet<(String, String)> = BTreeSet::new();
    let flake_path = repo.root.join("flake.nix");
    let flake_text = std::fs::read_to_string(&flake_path)
        .with_context(|| fill(m::READING, &[("path", &flake_path.display().to_string())]))?;

    for (host, f) in &facts.hosts {
        let entries = host_entry_modules(host, &flake_text, repo);
        let (nodes, edges) = build_import_graph(&entries, repo);
        for n in &nodes {
            tree.add_file(n);
        }
        for (a, b) in &edges {
            import_edges.insert((d2_path(a), d2_path(b)));
        }
        for e in &entries {
            host_edges.push((sanitize(host), d2_path(&rel_str(e, repo))));
        }

        let (services, programs) = match f {
            Host::Nixos(n) => (&n.services, &n.programs),
            Host::Darwin(d) => (&d.services, &d.programs),
        };
        for (units, field) in [(services, 0), (programs, 1)] {
            for item in units {
                for sf in &item.files {
                    let Some(rel) = rel_from_store(sf) else {
                        continue;
                    };
                    if !repo.root.join(rel).exists() {
                        continue;
                    }
                    let mut rel = rel.to_string();
                    if repo.root.join(&rel).is_dir() {
                        rel = format!("{rel}/default.nix");
                        if !repo.root.join(&rel).exists() {
                            continue;
                        }
                    }
                    let meta = tree.add_file(&rel);
                    let list = if field == 0 {
                        &mut meta.svcs
                    } else {
                        &mut meta.progs
                    };
                    if !list.contains(&item.name) {
                        list.push(item.name.clone());
                    }
                }
            }
        }
    }

    let edge = |(from, to): &(String, String)| fill(t::EDGE, &[("from", from), ("to", to)]);
    let mut o = preamble(style);
    o.push(String::new());
    for host in facts.hosts.keys() {
        o.push(fill(t::HOST, &[("id", &sanitize(host)), ("host", host)]));
    }
    o.push(String::new());
    tree.emit(&mut o, 0);
    o.push(String::new());
    o.push(t::HOST_EDGES.into());
    o.extend(host_edges.iter().map(edge));
    o.push(String::new());
    o.push(t::IMPORT_EDGES.into());
    o.extend(import_edges.iter().map(edge));

    write_and_render(out, "modules", &o, render_svg, style)
}
