use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub const MARKER_WORD: &str = "Auto-generated";

pub const MD_MARKER: &str = "<!-- Auto-generated from the Nix config by nixdiag. Do not edit. -->";

pub const JSON_MARKER: &str = "Auto-generated from the Nix config by nixdiag. Do not edit.";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WKind {
    Auto,
    Once,
    Svg,
    Extra,
    Volatile,
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

    pub fn write_auto(&mut self, rel: &Path, text: &str) -> Result<()> {
        let path = self.guard(rel)?;
        write_text(&path, text)?;
        self.record(rel, WKind::Auto);
        Ok(())
    }

    pub fn write_once(&mut self, rel: &Path, text: &str) -> Result<()> {
        let path = self.root.join(rel);
        self.record(rel, WKind::Once);
        if path.exists() {
            return Ok(());
        }
        write_text(&path, text)
    }

    pub fn write_json<T: serde::Serialize>(
        &mut self,
        rel: &Path,
        value: &T,
        kind: WKind,
    ) -> Result<()> {
        let path = self.guard(rel)?;
        let body = serde_json::to_string_pretty(value)
            .with_context(|| format!("serializing {}", rel.display()))?;
        debug_assert!(
            body.contains(MARKER_WORD),
            "{} serialized without the {MARKER_WORD} marker; it could not be \
             regenerated over itself",
            rel.display()
        );
        write_text(&path, &body)?;
        self.record(rel, kind);
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("nixdiag-out-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn every_marker_satisfies_the_guard() {
        assert!(MD_MARKER.contains(MARKER_WORD));
        assert!(JSON_MARKER.contains(MARKER_WORD));
    }

    #[test]
    fn hand_written_files_are_never_clobbered() {
        let dir = scratch("handwritten");
        let out = Out::new(dir.clone());
        fs::write(dir.join("notes.md"), "mine, written by hand\n").unwrap();
        assert!(out.guard(Path::new("notes.md")).is_err());
        fs::write(dir.join("gen.md"), MD_MARKER).unwrap();
        assert!(out.guard(Path::new("gen.md")).is_ok());
    }

    #[test]
    fn json_written_once_can_be_written_again() {
        #[derive(serde::Serialize)]
        struct Doc {
            meta: Meta,
        }
        #[derive(serde::Serialize)]
        struct Meta {
            generator: &'static str,
        }
        let doc = Doc {
            meta: Meta {
                generator: JSON_MARKER,
            },
        };

        let dir = scratch("json-rewrite");
        let rel = Path::new("api/v1/thing.json");
        let mut out = Out::new(dir.clone());
        out.write_json(rel, &doc, WKind::Auto).unwrap();
        out.write_json(rel, &doc, WKind::Auto).unwrap();

        let body = fs::read_to_string(dir.join(rel)).unwrap();
        assert!(body.contains(MARKER_WORD));
        assert_eq!(out.manifest.len(), 2);
        assert_eq!(out.manifest[0].kind, WKind::Auto);
    }

    #[test]
    fn volatile_is_not_auto() {
        assert_ne!(WKind::Volatile, WKind::Auto);
        assert_ne!(WKind::Volatile, WKind::Svg);
    }
}
