use crate::text::{fill, messages as m};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Out {
    pub root: PathBuf,
}

impl Out {
    pub fn new(root: PathBuf) -> Self {
        Out { root }
    }

    pub fn write(&self, rel: &Path, text: &str) -> Result<()> {
        let path = self.root.join(rel);
        let shown = path.display().to_string();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, format!("{}\n", text.trim_end()))
            .with_context(|| fill(m::WRITING, &[("path", &shown)]))?;
        println!("{}", fill(m::WROTE, &[("path", &shown)]));
        Ok(())
    }
}
