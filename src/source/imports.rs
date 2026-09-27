use super::repo::Repo;
use crate::conf::repo::{DEFAULT, ENTRY_KEYS, HOSTS};
use regex::Regex;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;

static IMPORTS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"imports\s*=").unwrap());
static TOKEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\.\.?/[^\s\]"';]+"#).unwrap());
static IMPORT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\bimport\s+(\.\.?/[^\s\])"';]+)"#).unwrap());
static ENTRIES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    ENTRY_KEYS
        .iter()
        .map(|k| Regex::new(&format!(r"{k}\s*=\s*(\.\S+?)\s*;")).unwrap())
        .collect()
});

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
        p.push(DEFAULT);
    } else if p.extension().map(|e| e != "nix").unwrap_or(true) {
        p.set_extension("nix");
    }
    p
}

pub fn host_entry_modules(host: &str, flake_text: &str, repo: &Repo) -> Vec<PathBuf> {
    let block_re = Regex::new(&format!(
        r"(?s)\b{}\s*=\s*\{{(.*?)\n\s*\}};",
        regex::escape(host)
    ))
    .unwrap();
    let block = block_re
        .captures(flake_text)
        .and_then(|c| c.get(1))
        .map_or("", |m| m.as_str());
    let mut files: Vec<PathBuf> = ENTRIES
        .iter()
        .filter_map(|re| re.captures(block))
        .map(|m| with_nix_ext(normalize(&repo.root.join(&m[1]))))
        .filter(|p| p.exists())
        .collect();
    if files.is_empty() {
        let cand = repo.root.join(HOSTS).join(host).join(DEFAULT);
        if cand.exists() {
            files.push(cand);
        }
    }
    files
}

fn parse_imports(nix_file: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(nix_file) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for m in IMPORTS.find_iter(&text) {
        let seg = &text[m.end()..];
        let seg = seg.split(';').next().unwrap_or(seg);
        out.extend(TOKEN.find_iter(seg).map(|t| t.as_str().to_string()));
    }
    out.extend(IMPORT.captures_iter(&text).map(|c| c[1].to_string()));
    out
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
        let rf = repo.rel(&f);
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
            let rc = repo.rel(&child);
            nodes.insert(rc.clone());
            edges.insert((rf.clone(), rc));
            stack.push(child);
        }
    }
    (nodes, edges)
}
