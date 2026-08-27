//! The module import graph of the documented repo.
//!
//! Follows both `imports = [ ... ]` lists and plain `import ./path`
//! expressions, starting from each host's entry module. Two consumers: the
//! module-tree diagram draws it, and annotation attachment uses it to decide
//! which hosts a given file reaches.

use super::repo::Repo;
use regex::Regex;
use std::collections::HashSet;
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

/// Relative-path tokens appearing in `imports = [ ... ];` lists, plus the
/// argument of plain `import ./path` expressions (files pulled in as data,
/// e.g. an upstream table, are part of the host's assembly too).
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
    let expr_re = Regex::new(r#"\bimport\s+(\.\.?/[^\s\])"';]+)"#).unwrap();
    for c in expr_re.captures_iter(&text) {
        out.push(c[1].to_string());
    }
    out
}

pub fn rel_str(p: &Path, repo: &Repo) -> String {
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
