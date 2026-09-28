use super::{label, Cloud, Flow, Target, View};
use crate::conf::files::{diagram, page};
use crate::render::d2::{Class, Diagram, Doc};
use crate::text::d2::topology as t;
use indexmap::IndexMap;
use itertools::Itertools;

pub struct Overview<'v, 'a>(pub &'v View<'a>);

impl Overview<'_, '_> {
    fn bare_rows(&self, host: &str) -> Vec<(&'static str, String)> {
        let Some(n) = self.0.facts.hosts.get(host).and_then(|h| h.as_nixos()) else {
            return Vec::new();
        };
        [(t::TCP, &n.tcp), (t::UDP, &n.udp)]
            .into_iter()
            .filter(|(_, ports)| !ports.is_empty())
            .map(|(proto, ports)| (proto, ports.iter().join(t::PORTS_SEP)))
            .collect()
    }
}

impl Diagram for Overview<'_, '_> {
    fn stem(&self) -> String {
        diagram::TOPOLOGY.into()
    }

    fn draw(&self, doc: &mut Doc) {
        let v = self.0;
        doc.vertical();
        let mut clouds: IndexMap<String, Cloud> =
            v.ingress.iter().map(|i| (i.net.key(), i.net)).collect();
        let mut pairs: IndexMap<(&str, &str), Vec<Flow>> = IndexMap::new();
        let mut outbound = Vec::new();
        for f in v.flows() {
            match (f.from.host(), &f.to) {
                (_, Target::Net(c)) => {
                    clouds.insert(c.key(), *c);
                    outbound.push(f);
                }
                (Some(a), Target::Node(b, _)) if a != *b => {
                    pairs.entry((a, *b)).or_default().push(f);
                }
                _ => {}
            }
        }
        v.boxes(doc, v.hosts.keys().copied());
        for c in clouds.values() {
            c.draw(doc, v.home(&Target::Net(*c)).as_deref());
        }
        for (host, units) in &v.hosts {
            let home = v.home(&Target::Node(host, None));
            let table = doc.place(home.as_deref(), host, &v.label(host), Class::Table);
            table.link(&page::host(host));
            for u in units.values() {
                let ports = u.ports().unwrap_or(t::NO_PORTS.into());
                table.row(u.name, &ports, u.role.as_deref());
            }
            if v.facts.bare() {
                for (proto, ports) in self.bare_rows(host) {
                    table.row(proto, &ports, None);
                }
            }
        }
        for i in &v.ingress {
            doc.edge(
                &v.placed(&Target::Net(i.net)),
                &v.placed(&i.node),
                label(&i.label),
                i.net.edge(),
            );
        }
        for f in &outbound {
            if let Target::Net(c) = &f.to {
                doc.edge(
                    &v.placed(&f.from),
                    &v.placed(&f.to),
                    label(f.label),
                    c.edge(),
                );
            }
        }
        for ((a, b), flows) in &pairs {
            match flows.as_slice() {
                [f] => doc.edge(
                    &v.placed(&f.from),
                    &v.placed(&f.to),
                    label(f.label),
                    Class::Flow,
                ),
                _ => doc.edge(
                    &v.placed(&Target::Node(a, None)),
                    &v.placed(&Target::Node(b, None)),
                    Some(&t::flows(flows.len())),
                    Class::Flow,
                ),
            }
        }
    }
}
