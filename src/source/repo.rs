use std::path::PathBuf;

pub struct Repo {
    pub root: PathBuf,
}

pub fn rel_from_store(path: &str) -> Option<&str> {
    let marker = "-source/";
    path.find(marker).map(|i| &path[i + marker.len()..])
}

impl Repo {
    pub fn new(root: PathBuf) -> Self {
        Repo { root }
    }

    pub fn repo_files(&self, store_files: &[String]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for f in store_files {
            let Some(rel) = rel_from_store(f) else {
                continue;
            };
            let mut rel = rel.to_string();
            if self.root.join(&rel).is_dir() {
                rel = format!("{rel}/default.nix");
            }
            if self.root.join(&rel).exists() && !out.contains(&rel) {
                out.push(rel);
            }
        }
        out
    }
}
