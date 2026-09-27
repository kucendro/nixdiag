use super::dot::{card, id, Diagram, Dot, Graph, Paint};
use crate::conf::files::diagram;
use crate::conf::palette::{diagram as p, Color};
use crate::facts::Facts;
use crate::source::imports::{build_import_graph, host_entry_modules};
use crate::source::repo::Repo;
use anyhow::Result;
use dot_writer::Attributes;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Shape {
    Service,
    Program,
}

impl Shape {
    fn fill(self) -> Color {
        match self {
            Shape::Service => p::APP_FILL,
            Shape::Program => p::PROG_FILL,
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

    fn emit(&self, g: &mut Graph, dot: &Dot, prefix: &str) {
        for (name, sub) in &self.dirs {
            let mut c = g.cluster();
            c.text(&[name])
                .stroke(dot.color(&p::BASE_STROKE))
                .set("style", "rounded,dashed", true);
            sub.emit(&mut c, dot, &format!("{prefix}{name}/"));
        }
        let (fill, stroke) = (dot.color(&p::BASE_FILL), dot.color(&p::BASE_STROKE));
        for (name, units) in &self.files {
            let rows: Vec<(&str, &str)> = units
                .iter()
                .map(|(shape, unit)| (dot.color(&shape.fill()), unit.as_str()))
                .collect();
            g.node_named(id(&format!("{prefix}{name}")))
                .shape("plain")
                .set_html(&card(name, fill, stroke, &rows));
        }
    }
}

pub struct Modules {
    hosts: Vec<String>,
    tree: Tree,
    edges: Vec<(String, String)>,
}

impl Modules {
    pub fn new(facts: &Facts, repo: &Repo) -> Result<Self> {
        let mut tree = Tree::default();
        let mut edges: Vec<(String, String)> = Vec::new();
        let mut imports: BTreeSet<(String, String)> = BTreeSet::new();
        let flake_text = repo.flake()?;

        for (host, f) in &facts.hosts {
            let entries = host_entry_modules(host, &flake_text, repo);
            let (nodes, found) = build_import_graph(&entries, repo);
            for n in &nodes {
                tree.add_file(n);
            }
            imports.extend(found);
            edges.extend(entries.iter().map(|e| (host.clone(), repo.rel(e))));

            let b = f.base();
            for (shape, units) in [(Shape::Service, &b.services), (Shape::Program, &b.programs)] {
                for item in units {
                    for rel in item.files.iter().filter_map(|f| repo.file(f)) {
                        tree.add_file(&rel).insert((shape, item.name.clone()));
                    }
                }
            }
        }
        edges.extend(imports);
        let hosts = facts.hosts.keys().cloned().collect();
        Ok(Modules { hosts, tree, edges })
    }
}

impl Diagram for Modules {
    fn stem(&self) -> &'static str {
        diagram::MODULES
    }

    fn draw(&self, g: &mut Graph, dot: &Dot) {
        for host in &self.hosts {
            g.node_named(id(host))
                .bold(host)
                .shape("ellipse")
                .fill(dot.color(&p::HOST_CLOUD))
                .stroke(dot.color(&p::HOST_STROKE));
        }
        self.tree.emit(g, dot, "");
        for (from, to) in &self.edges {
            g.edge(id(from), id(to));
        }
    }
}
