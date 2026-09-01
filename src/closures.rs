//! Per-host system closure sizes — the second data input to `render`.
//!
//! Deliberately not part of `facts`: facts are pure evaluation, while nar
//! sizes exist only for *realised* store paths. Keeping the provenances apart
//! is what lets `facts.json` stay schema 2 and `mkFacts` stay a pure eval.
//!
//! Totals and counts are always derived from `paths`, never stored, so they
//! cannot drift out of agreement with the list they summarise.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Bump on any breaking change to this model or to nix/closures.nix.
pub const CLOSURES_SCHEMA: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct Closures {
    pub schema: u32,
    pub hosts: IndexMap<String, HostClosure>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HostClosure {
    pub paths: Vec<ClosurePath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosurePath {
    pub path: String,
    pub nar_size: u64,
}

impl HostClosure {
    pub fn total(&self) -> u64 {
        self.paths.iter().map(|p| p.nar_size).sum()
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// The `n` biggest contributors, largest first. Ties break on path so the
    /// rendered table is stable.
    pub fn largest(&self, n: usize) -> Vec<&ClosurePath> {
        let mut v: Vec<&ClosurePath> = self.paths.iter().collect();
        v.sort_by(|a, b| b.nar_size.cmp(&a.nar_size).then(a.path.cmp(&b.path)));
        v.truncate(n);
        v
    }
}

impl Closures {
    /// How many hosts hold each path, with its size. BTreeMap so every derived
    /// listing comes out in a deterministic order.
    fn occurrences(&self) -> BTreeMap<&str, (usize, u64)> {
        let mut seen: BTreeMap<&str, (usize, u64)> = BTreeMap::new();
        for host in self.hosts.values() {
            for p in &host.paths {
                let e = seen.entry(p.path.as_str()).or_insert((0, p.nar_size));
                e.0 += 1;
            }
        }
        seen
    }

    /// Paths every host carries — the common base.
    pub fn shared(&self) -> Vec<(&str, u64)> {
        let n = self.hosts.len();
        self.occurrences()
            .into_iter()
            .filter(|(_, (count, _))| *count == n)
            .map(|(p, (_, size))| (p, size))
            .collect()
    }

    /// Paths this host carries and no other — what it actually costs beyond
    /// the shared base.
    pub fn unique(&self, host: &str) -> Vec<(&str, u64)> {
        let occ = self.occurrences();
        let Some(h) = self.hosts.get(host) else {
            return Vec::new();
        };
        let mut v: Vec<(&str, u64)> = h
            .paths
            .iter()
            .filter(|p| occ.get(p.path.as_str()).map(|(c, _)| *c) == Some(1))
            .map(|p| (p.path.as_str(), p.nar_size))
            .collect();
        v.sort();
        v
    }

    /// Distinct paths across the fleet, and their total size.
    pub fn deduped(&self) -> (usize, u64) {
        let occ = self.occurrences();
        (occ.len(), occ.values().map(|(_, size)| size).sum())
    }

    /// What the fleet would weigh without sharing: every host's closure summed
    /// as if it stood alone.
    pub fn naive_sum(&self) -> u64 {
        self.hosts.values().map(HostClosure::total).sum()
    }

    /// One host's closure split by how widely each path is held: carried by
    /// every measured host, by some of them, or by this host alone.
    ///
    /// The three always sum to that host's `total()`, which is what lets the
    /// fleet chart stack them into one bar. With a single measured host every
    /// path is trivially held by all of them, so the whole closure lands in
    /// `shared` and the chart draws a plain bar instead of a legend that
    /// would distinguish nothing.
    pub fn split(&self, host: &str) -> Split {
        let n = self.hosts.len();
        let occ = self.occurrences();
        let mut s = Split::default();
        let Some(h) = self.hosts.get(host) else {
            return s;
        };
        for p in &h.paths {
            match occ.get(p.path.as_str()).map(|(count, _)| *count) {
                Some(c) if c == n => s.shared += p.nar_size,
                Some(1) => s.unique += p.nar_size,
                _ => s.partial += p.nar_size,
            }
        }
        s
    }
}

/// See `Closures::split`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Split {
    pub shared: u64,
    pub partial: u64,
    pub unique: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(path: &str, nar_size: u64) -> ClosurePath {
        ClosurePath {
            path: path.into(),
            nar_size,
        }
    }

    /// luna and sol share `libc` and `bash`; each has one path of its own.
    fn fixture() -> Closures {
        let mut hosts = IndexMap::new();
        hosts.insert(
            "luna".to_string(),
            HostClosure {
                paths: vec![p("libc", 100), p("bash", 50), p("nginx", 10)],
            },
        );
        hosts.insert(
            "sol".to_string(),
            HostClosure {
                paths: vec![p("libc", 100), p("bash", 50), p("postgres", 400)],
            },
        );
        Closures { schema: 1, hosts }
    }

    #[test]
    fn totals_are_derived_from_the_path_list() {
        let c = fixture();
        assert_eq!(c.hosts["luna"].total(), 160);
        assert_eq!(c.hosts["luna"].len(), 3);
    }

    #[test]
    fn shared_is_what_every_host_carries() {
        assert_eq!(fixture().shared(), vec![("bash", 50), ("libc", 100)]);
    }

    #[test]
    fn unique_excludes_anything_another_host_also_has() {
        let c = fixture();
        assert_eq!(c.unique("luna"), vec![("nginx", 10)]);
        assert_eq!(c.unique("sol"), vec![("postgres", 400)]);
        assert_eq!(c.unique("nope"), vec![]);
    }

    #[test]
    fn deduplication_counts_a_shared_path_once() {
        let c = fixture();
        // libc + bash + nginx + postgres = 4 paths, 560 bytes
        assert_eq!(c.deduped(), (4, 560));
        // 160 + 550, i.e. the shared 150 counted twice
        assert_eq!(c.naive_sum(), 710);
    }

    #[test]
    fn largest_is_size_descending_and_bounded() {
        let c = fixture();
        let top = c.hosts["sol"].largest(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].path, "postgres");
        assert_eq!(top[1].path, "libc");
    }

    #[test]
    fn a_split_partitions_the_host_total() {
        let c = fixture();
        // Two hosts, so nothing can be held by "some but not all".
        assert_eq!(
            c.split("luna"),
            Split {
                shared: 150,
                partial: 0,
                unique: 10
            }
        );
        assert_eq!(
            c.split("luna").shared + c.split("luna").unique,
            c.hosts["luna"].total()
        );
        assert_eq!(c.split("nope"), Split::default());
    }

    #[test]
    fn a_third_host_makes_the_partial_band_possible() {
        let mut c = fixture();
        // `nginx` now sits on two of three hosts: shared by some, unique to
        // none, and counted in neither of the other two bands.
        c.hosts.insert(
            "terra".to_string(),
            HostClosure {
                paths: vec![p("libc", 100), p("bash", 50), p("nginx", 10)],
            },
        );
        assert_eq!(
            c.split("luna"),
            Split {
                shared: 150,
                partial: 10,
                unique: 0
            }
        );
        assert_eq!(
            c.split("sol"),
            Split {
                shared: 150,
                partial: 0,
                unique: 400
            }
        );
    }

    #[test]
    fn a_single_host_shares_everything_with_itself() {
        let mut hosts = IndexMap::new();
        hosts.insert(
            "only".to_string(),
            HostClosure {
                paths: vec![p("libc", 100)],
            },
        );
        let c = Closures { schema: 1, hosts };
        // Degenerate but consistent; the page suppresses the fleet section.
        assert_eq!(c.shared(), vec![("libc", 100)]);
        assert_eq!(c.deduped(), (1, 100));
        // "shared" wins the tie, so the split still sums to the total.
        assert_eq!(
            c.split("only"),
            Split {
                shared: 100,
                partial: 0,
                unique: 0
            }
        );
    }
}
