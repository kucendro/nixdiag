//! Module-tree diagram — port of gen-diagram.py.

use crate::d2::{write_and_render, D2_HEADER};
use crate::facts::{Facts, Host};
use crate::output::Out;
use crate::repo::{rel_from_store, Repo};
use crate::util::sanitize;
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Component, Path, PathBuf};

fn import_token_re() -> Regex {
    Regex::new(r#"\.\.?/[^\s\]"';]+"#).unwrap()
}

/// Lexical normalization (Python Path.resolve without symlink following).
fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

fn with_nix_ext(mut p: PathBuf) -> PathBuf {
    if p.is_dir() {
        p.push("default.nix");
    } else if p.extension().map(|e| e != "nix").unwrap_or(true) {
        p.set_extension("nix");
    }
    p
}

/// Entry module files for a host, from targetModule/hardwareModule keys in
/// flake.nix, with hosts/<name>/default.nix as the convention fallback.
pub fn host_entry_modules(host: &str, flake_text: &str, repo: &Repo) -> Vec<PathBuf> {
    let block_re = Regex::new(&format!(
        r"(?s)\b{}\s*=\s*\{{(.*?)\n\s*\}};",
        regex::escape(host)
    ))
    .unwrap();
    let block = block_re
        .captures(flake_text)
        .map(|c| c[1].to_string())
        .unwrap_or_default();
    let mut files = Vec::new();
    for key in ["targetModule", "hardwareModule"] {
        let key_re = Regex::new(&format!(r"{key}\s*=\s*(\.\S+?)\s*;")).unwrap();
        if let Some(m) = key_re.captures(&block) {
            let p = with_nix_ext(normalize(&repo.root.join(&m[1])));
            if p.exists() {
                files.push(p);
            }
        }
    }
    if files.is_empty() {
        let cand = repo.root.join("hosts").join(host).join("default.nix");
        if cand.exists() {
            files.push(cand);
        }
    }
    files
}

/// Relative-path tokens appearing in `imports = [ ... ];` lists.
fn parse_imports(nix_file: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(nix_file) else {
        return Vec::new();
    };
    let imports_re = Regex::new(r"imports\s*=").unwrap();
    let token_re = import_token_re();
    let mut out = Vec::new();
    for m in imports_re.find_iter(&text) {
        let seg = &text[m.end()..];
        let seg = seg.split(';').next().unwrap_or(seg);
        for t in token_re.find_iter(seg) {
            out.push(t.as_str().to_string());
        }
    }
    out
}

fn rel_str(p: &Path, repo: &Repo) -> String {
    match p.strip_prefix(&repo.root) {
        Ok(r) => r.to_string_lossy().replace('\\', "/"),
        Err(_) => p.to_string_lossy().into_owned(),
    }
}

pub fn build_import_graph(
    entries: &[PathBuf],
    repo: &Repo,
) -> (HashSet<String>, HashSet<(String, String)>) {
    let mut nodes = HashSet::new();
    let mut edges = HashSet::new();
    let mut seen = HashSet::new();
    let mut stack: Vec<PathBuf> = entries.to_vec();
    while let Some(f) = stack.pop() {
        let rf = rel_str(&f, repo);
        if !seen.insert(rf.clone()) {
            continue;
        }
        nodes.insert(rf.clone());
        for tok in parse_imports(&f) {
            let base = f.parent().unwrap_or(Path::new("."));
            let child = with_nix_ext(normalize(&base.join(&tok)));
            if !child.exists() {
                continue;
            }
            let rc = rel_str(&child, repo);
            nodes.insert(rc.clone());
            edges.insert((rf.clone(), rc));
            stack.push(child);
        }
    }
    (nodes, edges)
}

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
        for (name, sub) in &self.dirs {
            out.push(format!("{pad}{}: \"{name}\" {{", sanitize(name)));
            sub.emit(out, indent + 1);
            out.push(format!("{pad}}}"));
        }
        for (fname, meta) in &self.files {
            let fid = sanitize(fname);
            if meta.svcs.is_empty() && meta.progs.is_empty() {
                out.push(format!("{pad}{fid}: \"{fname}\" {{ shape: page }}"));
                continue;
            }
            out.push(format!("{pad}{fid}: \"{fname}\" {{ shape: page"));
            let mut svcs = meta.svcs.clone();
            svcs.sort();
            for s in svcs {
                out.push(format!(
                    "{pad}  svc_{}: \"{s}\" {{ shape: oval; style.fill: ${{appFill}} }}",
                    sanitize(&s)
                ));
            }
            let mut progs = meta.progs.clone();
            progs.sort();
            for p in progs {
                out.push(format!(
                    "{pad}  prog_{}: \"{p}\" {{ shape: hexagon; style.fill: ${{progFill}} }}",
                    sanitize(&p)
                ));
            }
            out.push(format!("{pad}}}"));
        }
    }
}

pub fn generate(
    facts: &Facts,
    repo: &Repo,
    out: &mut Out,
    render_svg: bool,
    style: &crate::d2::D2Style,
) -> Result<()> {
    let mut tree = Tree::default();
    let mut host_edges: Vec<(String, String)> = Vec::new();
    let mut import_edges: BTreeSet<(String, String)> = BTreeSet::new();
    let flake_path = repo.root.join("flake.nix");
    let flake_text = std::fs::read_to_string(&flake_path)
        .with_context(|| format!("reading {}", flake_path.display()))?;

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
            host_edges.push((host.clone(), d2_path(&rel_str(e, repo))));
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

    let mut o: Vec<String> = D2_HEADER.iter().map(|s| s.to_string()).collect();
    o.extend(crate::d2::vars_block(style));
    o.push("direction: right".into());
    o.push(String::new());
    for host in facts.hosts.keys() {
        o.push(format!(
            "{}: \"{host}\" {{ shape: cloud; style.fill: ${{hostCloud}}; style.bold: true }}",
            sanitize(host)
        ));
    }
    o.push(String::new());
    tree.emit(&mut o, 0);
    o.push(String::new());
    o.push("# host -> entry module".into());
    for (host, fid) in &host_edges {
        o.push(format!("{} -> {fid}", sanitize(host)));
    }
    o.push(String::new());
    o.push("# module imports".into());
    for (a, b) in &import_edges {
        o.push(format!("{a} -> {b}"));
    }

    write_and_render(out, "modules", &o, render_svg, style)
}
