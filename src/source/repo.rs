use crate::conf::repo::{DEFAULT, FLAKE, STORE_SOURCE};
use crate::text::{fill, messages as m};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct Repo {
    pub root: PathBuf,
}

impl Repo {
    pub fn new(root: PathBuf) -> Self {
        Repo { root }
    }

    pub fn flake(&self) -> Result<String> {
        let path = self.root.join(FLAKE);
        std::fs::read_to_string(&path)
            .with_context(|| fill(m::READING, &[("path", &path.display().to_string())]))
    }

    pub fn file(&self, store_path: &str) -> Option<String> {
        let rel = &store_path[store_path.find(STORE_SOURCE)? + STORE_SOURCE.len()..];
        let rel = if self.root.join(rel).is_dir() {
            format!("{rel}/{DEFAULT}")
        } else {
            rel.to_string()
        };
        self.root.join(&rel).exists().then_some(rel)
    }

    pub fn files(&self, store_paths: &[String]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for rel in store_paths.iter().filter_map(|p| self.file(p)) {
            if !out.contains(&rel) {
                out.push(rel);
            }
        }
        out
    }

    pub fn rel(&self, p: &Path) -> String {
        match p.strip_prefix(&self.root) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => p.to_string_lossy().into_owned(),
        }
    }
}
