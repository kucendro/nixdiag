use crate::text::messages::{self as m, Fail};
use anyhow::{Context, Result};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct Out {
    pub root: PathBuf,
}

impl Out {
    pub fn new(root: PathBuf) -> Self {
        Out { root }
    }

    pub fn sub(&self, rel: &str) -> Out {
        Out::new(self.root.join(rel))
    }

    pub fn write(&self, rel: impl AsRef<Path>, text: &str) -> Result<()> {
        self.put(rel, |p| fs::write(p, format!("{}\n", text.trim_end())))
    }

    pub fn copy(&self, from: &Path, rel: impl AsRef<Path>) -> Result<()> {
        self.put(rel, |p| fs::copy(from, p).map(drop))
    }

    pub fn mirror(&self, from: &Out, name: &str) -> Result<()> {
        let path = from.root.join(name);
        if path.exists() {
            self.copy(&path, name)?;
        }
        Ok(())
    }

    pub fn put(
        &self,
        rel: impl AsRef<Path>,
        f: impl FnOnce(&Path) -> io::Result<()>,
    ) -> Result<()> {
        let path = self.root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        f(&path).with_context(|| Fail::Writing(path.clone()))?;
        println!("{}", m::wrote(path.display()));
        Ok(())
    }
}
