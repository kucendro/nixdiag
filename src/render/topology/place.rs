use super::{Cloud, Target, View};
use crate::facts::Scope;
use crate::render::d2::{Class, Doc};
use crate::text::d2::topology as t;
use crate::topology::Network;
use indexmap::IndexSet;

fn key(location: &str) -> String {
    format!("location {location}")
}

impl<'a> View<'a> {
    fn location(&self, host: &str) -> Option<&'a str> {
        self.locations.get(host).map(String::as_str)
    }

    fn settled(&self, n: &'a Network) -> Option<&'a str> {
        if n.location.is_some() {
            return n.location.as_deref();
        }
        if !matches!(n.kind, Some(Scope::Lan | Scope::Public)) {
            return None;
        }
        let homes: IndexSet<_> = n.members.iter().map(|m| self.location(&m.host)).collect();
        homes
            .first()
            .copied()
            .flatten()
            .filter(|_| homes.len() == 1)
    }

    pub fn home(&self, target: &Target) -> Option<String> {
        let location = match target {
            Target::Node(h, _) => self.location(h),
            Target::Net(Cloud::Network(n)) => self.settled(n),
            Target::Net(Cloud::Net(_)) => None,
        };
        location.map(key)
    }

    pub fn placed(&self, target: &Target) -> Vec<String> {
        self.home(target).into_iter().chain(target.path()).collect()
    }

    pub fn boxes<'h>(&self, doc: &mut Doc, hosts: impl IntoIterator<Item = &'h str>) {
        let used: IndexSet<&str> = hosts.into_iter().filter_map(|h| self.location(h)).collect();
        for l in used {
            doc.shape(&key(l), &t::location(l), Class::Location);
        }
    }
}
