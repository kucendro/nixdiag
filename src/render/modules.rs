use super::d2::{Class, Diagram, Doc};
use crate::conf::files::diagram;
use crate::facts::Facts;
use crate::source::imports::{build_import_graph, host_entry_modules};
use crate::source::repo::Repo;
use crate::text::d2::modules as t;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Service,
    Program,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Kind::Service => t::SERVICE,
            Kind::Program => t::PROGRAM,
        }
    }
}

pub struct Modules {
    host: String,
    files: BTreeMap<String, BTreeSet<(Kind, String)>>,
    entries: BTreeSet<String>,
    imports: BTreeSet<(String, String)>,
}

impl Modules {
    pub fn per_host(facts: &Facts, repo: &Repo) -> Result<Vec<Self>> {
        let flake = repo.flake()?;
        let boards = facts.hosts.iter().map(|(host, f)| {
            let entries = host_entry_modules(host, &flake, repo);
            let (nodes, imports) = build_import_graph(&entries, repo);
            let mut files: BTreeMap<String, BTreeSet<(Kind, String)>> =
                nodes.into_iter().map(|n| (n, BTreeSet::new())).collect();
            let b = f.base();
            for (kind, units) in [(Kind::Service, &b.services), (Kind::Program, &b.programs)] {
                for u in units {
                    for rel in u.files.iter().filter_map(|f| repo.file(f)) {
                        files.entry(rel).or_default().insert((kind, u.name.clone()));
                    }
                }
            }
            Modules {
                host: host.clone(),
                files,
                entries: entries.iter().map(|e| repo.rel(e)).collect(),
                imports: imports.into_iter().collect(),
            }
        });
        Ok(boards.collect())
    }
}

impl Diagram for Modules {
    fn stem(&self) -> String {
        diagram::modules(&self.host)
    }

    fn draw(&self, doc: &mut Doc) {
        doc.shape(&self.host, &self.host, Class::Host);
        for (file, units) in &self.files {
            let table = doc.shape(file, file, Class::Table);
            for (kind, unit) in units {
                table.row(unit, kind.label(), None);
            }
        }
        for entry in &self.entries {
            doc.edge(&[&self.host], &[entry], None, Class::Arrow);
        }
        for (from, to) in &self.imports {
            doc.edge(&[from], &[to], None, Class::Arrow);
        }
    }
}
