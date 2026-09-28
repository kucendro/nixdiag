use super::{label, Flow, Net, Target, View};
use crate::conf::files::{diagram, page};
use crate::render::d2::{Class, Diagram, Doc};
use crate::text::d2::topology as t;
use std::collections::{BTreeMap, BTreeSet};
use std::iter::once;

pub struct HostBoard<'v, 'a> {
    pub view: &'v View<'a>,
    pub host: &'a str,
}

impl HostBoard<'_, '_> {
    fn local(&self, target: &Target) -> bool {
        target.host() == Some(self.host)
    }

    fn path(&self, target: &Target) -> Vec<String> {
        match target {
            Target::Node(h, Some(u)) if *h != self.host => vec![format!("{h}/{u}")],
            _ => target.path().into_iter().map(String::from).collect(),
        }
    }
}

impl Diagram for HostBoard<'_, '_> {
    fn stem(&self) -> String {
        diagram::topology(self.host)
    }

    fn draw(&self, doc: &mut Doc) {
        let v = self.view;
        doc.vertical();
        let machine = doc.shape(self.host, &v.label(self.host), Class::Machine);
        for u in v.hosts[self.host].values() {
            let ports = u.ports();
            let lines: Vec<&str> = once(u.name)
                .chain(u.role.as_deref())
                .chain(ports.as_deref())
                .collect();
            let node = machine.child(u.name, &t::unit(&lines), u.class());
            if let Some(d) = u.description {
                node.tooltip(d.trim());
            }
        }

        let ingress: Vec<_> = v.ingress.iter().filter(|i| self.local(&i.node)).collect();
        let mut nets: BTreeSet<Net> = ingress.iter().map(|i| i.net).collect();
        let mut ghosts: BTreeMap<String, (String, String)> = BTreeMap::new();
        let mut edges = Vec::new();
        for Flow {
            from,
            to,
            label: text,
        } in v.flows()
        {
            if !self.local(&from) && !self.local(&to) {
                continue;
            }
            let class = match (&from, &to) {
                (_, Target::Net(n)) => {
                    nets.insert(*n);
                    n.edge()
                }
                _ if self.local(&from) && self.local(&to) => Class::Local,
                _ => Class::Flow,
            };
            for end in [&from, &to] {
                if let Target::Node(h, u) = end {
                    if *h != self.host {
                        let text = u.map_or(h.to_string(), |u| t::ghost(h, u));
                        ghosts.insert(self.path(end).concat(), (text, page::host(h)));
                    }
                }
            }
            edges.push((self.path(&from), self.path(&to), text, class));
        }

        for n in &nets {
            n.draw(doc);
        }
        for (key, (text, link)) in &ghosts {
            doc.shape(key, text, Class::Ghost).link(link);
        }
        for i in ingress {
            doc.edge(
                &[i.net.key()],
                &i.node.path(),
                label(&i.label),
                i.net.edge(),
            );
        }
        for (from, to, text, class) in &edges {
            let from: Vec<&str> = from.iter().map(String::as_str).collect();
            let to: Vec<&str> = to.iter().map(String::as_str).collect();
            doc.edge(&from, &to, label(text), *class);
        }
    }
}
