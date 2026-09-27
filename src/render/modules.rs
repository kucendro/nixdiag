use super::d2::{preamble, write_and_render};
use super::out::Out;
use super::style::Style;
use crate::conf::files::diagram;
use crate::facts::Facts;
use crate::source::imports::{build_import_graph, host_entry_modules};
use crate::source::repo::Repo;
use crate::text::d2::modules as t;
use crate::text::fill;
use crate::util::sanitize;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

fn d2_path(rel: &str) -> String {
    rel.split('/').map(sanitize).collect::<Vec<_>>().join(".")
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Shape {
    Service,
    Program,
}

impl Shape {
    fn template(self) -> &'static str {
        match self {
            Shape::Service => t::SERVICE,
            Shape::Program => t::PROGRAM,
        }
    }
}

#[derive(Default)]
struct Tree {
    dirs: BTreeMap<String, Tree>,
    files: BTreeMap<String, BTreeSet<(Shape, String)>>,
}

impl Tree {
    fn add_file(&mut self, rel: &str) -> &mut BTreeSet<(Shape, String)> {
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
        for (fname, units) in &self.files {
            if units.is_empty() {
                out.push(line(t::FILE, fname));
                continue;
            }
            out.push(line(t::FILE_OPEN, fname));
            for (shape, name) in units {
                out.push(line(shape.template(), name));
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
    style: &Style,
) -> Result<()> {
    let mut tree = Tree::default();
    let mut host_edges: Vec<(String, String)> = Vec::new();
    let mut import_edges: BTreeSet<(String, String)> = BTreeSet::new();
    let flake_text = repo.flake()?;

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
            host_edges.push((sanitize(host), d2_path(&repo.rel(e))));
        }

        let b = f.base();
        for (shape, units) in [(Shape::Service, &b.services), (Shape::Program, &b.programs)] {
            for item in units {
                for rel in item.files.iter().filter_map(|f| repo.file(f)) {
                    tree.add_file(&rel).insert((shape, item.name.clone()));
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

    write_and_render(out, diagram::MODULES, &o, render_svg, style)
}
