//! Guarded file writing plus the manifest of what a render produced.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Any generated file carries this word; the writer refuses to overwrite
/// an existing file that lacks it (i.e. anything hand-written).
pub const MARKER_WORD: &str = "Auto-generated";

pub const MD_MARKER: &str = "<!-- Auto-generated from the Nix config by nixdiag. Do not edit. -->";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WKind {
    /// Regenerated every run; participates in `nixdiag check`.
    Auto,
    /// Seeded once, then owned by the user (index.md, book.toml).
    Once,
    /// Derived binary-ish output (SVG); excluded from `check` diffs.
    Svg,
    /// User-supplied extra page copied into the wiki.
    Extra,
}

pub struct Written {
    pub rel: PathBuf,
    pub kind: WKind,
}

pub struct Out {
    pub root: PathBuf,
    pub manifest: Vec<Written>,
}

impl Out {
    pub fn new(root: PathBuf) -> Self {
        Out {
            root,
            manifest: Vec::new(),
        }
    }

    fn record(&mut self, rel: &Path, kind: WKind) {
        self.manifest.push(Written {
            rel: rel.to_path_buf(),
            kind,
        });
    }

    pub fn guard(&self, rel: &Path) -> Result<PathBuf> {
        let path = self.root.join(rel);
        if path.exists() {
            let existing = fs::read_to_string(&path).unwrap_or_default();
            if !existing.contains(MARKER_WORD) {
                bail!(
                    "refusing to overwrite {}: it exists but has no '{MARKER_WORD}' marker \
                     (looks hand-written); delete or move it first",
                    path.display()
                );
            }
        }
        Ok(path)
    }

    /// Write an auto-regenerated file (marker check applies).
    pub fn write_auto(&mut self, rel: &Path, text: &str) -> Result<()> {
        let path = self.guard(rel)?;
        write_text(&path, text)?;
        self.record(rel, WKind::Auto);
        Ok(())
    }

    /// Seed a user-owned file only if it does not exist yet.
    pub fn write_once(&mut self, rel: &Path, text: &str) -> Result<()> {
        let path = self.root.join(rel);
        self.record(rel, WKind::Once);
        if path.exists() {
            return Ok(());
        }
        write_text(&path, text)
    }

    pub fn record_svg(&mut self, rel: &Path) {
        self.record(rel, WKind::Svg);
    }

    pub fn record_extra(&mut self, rel: &Path) {
        self.record(rel, WKind::Extra);
    }
}

pub fn write_text(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let body = format!("{}\n", text.trim_end());
    fs::write(path, body).with_context(|| format!("writing {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}
