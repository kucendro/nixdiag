#[cfg(test)]
mod tests;

use crate::util::{package_name, store_name};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Closures {
    pub schema: u32,
    pub hosts: IndexMap<String, HostClosure>,
    #[serde(default)]
    pub served: Vec<String>,
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

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Total {
    pub paths: usize,
    pub size: u64,
}

impl<'a> FromIterator<&'a ClosurePath> for Total {
    fn from_iter<I: IntoIterator<Item = &'a ClosurePath>>(iter: I) -> Self {
        iter.into_iter().fold(Total::default(), |t, p| Total {
            paths: t.paths + 1,
            size: t.size + p.nar_size,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Share {
    pub name: String,
    pub size: u64,
    pub holders: usize,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Split {
    pub shared: u64,
    pub partial: u64,
    pub unique: u64,
}

impl HostClosure {
    pub fn total(&self) -> Total {
        self.paths.iter().collect()
    }

    pub fn largest(&self, n: usize) -> Vec<&ClosurePath> {
        let mut v: Vec<&ClosurePath> = self.paths.iter().collect();
        v.sort_by(|a, b| b.nar_size.cmp(&a.nar_size).then(a.path.cmp(&b.path)));
        v.truncate(n);
        v
    }
}

impl Closures {
    fn holders(&self) -> BTreeMap<&str, (usize, &ClosurePath)> {
        let mut seen = BTreeMap::new();
        for p in self.hosts.values().flat_map(|h| &h.paths) {
            seen.entry(p.path.as_str()).or_insert((0, p)).0 += 1;
        }
        seen
    }

    fn paths(&self, host: &str) -> impl Iterator<Item = &ClosurePath> {
        self.hosts.get(host).into_iter().flat_map(|h| &h.paths)
    }

    pub fn shared(&self) -> Total {
        let n = self.hosts.len();
        let holders = self.holders().into_values();
        holders.filter(|(c, _)| *c == n).map(|(_, p)| p).collect()
    }

    pub fn unique(&self, host: &str) -> Total {
        let holders = self.holders();
        self.paths(host)
            .filter(|p| holders[p.path.as_str()].0 == 1)
            .collect()
    }

    pub fn deduped(&self) -> Total {
        self.holders().into_values().map(|(_, p)| p).collect()
    }

    pub fn naive_sum(&self) -> u64 {
        self.hosts.values().map(|h| h.total().size).sum()
    }

    fn path_shares(&self, host: &str) -> Vec<Share> {
        let holders = self.holders();
        self.paths(host)
            .map(|p| Share {
                name: p.path.clone(),
                size: p.nar_size,
                holders: holders[p.path.as_str()].0,
            })
            .collect()
    }

    pub fn package_shares(&self, host: &str) -> Vec<Share> {
        let paths = self.path_shares(host);
        let mut groups: BTreeMap<(&str, usize), u64> = BTreeMap::new();
        for p in &paths {
            *groups
                .entry((package_name(store_name(&p.name)), p.holders))
                .or_default() += p.size;
        }
        let mut v: Vec<Share> = groups
            .into_iter()
            .map(|((name, holders), size)| Share {
                name: name.into(),
                size,
                holders,
            })
            .collect();
        v.sort_by(|a, b| b.size.cmp(&a.size).then(a.name.cmp(&b.name)));
        v
    }

    pub fn split(&self, host: &str) -> Split {
        let n = self.hosts.len();
        let mut s = Split::default();
        for p in self.path_shares(host) {
            match p.holders {
                c if c == n => s.shared += p.size,
                1 => s.unique += p.size,
                _ => s.partial += p.size,
            }
        }
        s
    }

    #[cfg(test)]
    pub fn of(hosts: Vec<(&str, Vec<(&str, u64)>)>) -> Closures {
        let path = |(path, nar_size): (&str, u64)| ClosurePath {
            path: path.into(),
            nar_size,
        };
        Closures {
            schema: 1,
            hosts: hosts
                .into_iter()
                .map(|(h, ps)| {
                    let paths = ps.into_iter().map(path).collect();
                    (h.to_string(), HostClosure { paths })
                })
                .collect(),
            served: vec![],
        }
    }
}
