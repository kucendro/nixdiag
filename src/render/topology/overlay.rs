use super::{Cloud, View};
use crate::conf::files::{diagram, page};
use crate::facts::{Plane, Scope};
use crate::render::d2::{Class, Diagram, Doc};
use crate::text::d2::networks as t;
use indexmap::{IndexMap, IndexSet};

pub struct Overlay<'v, 'a>(pub &'v View<'a>);

impl Overlay<'_, '_> {
    pub fn drawn(&self) -> bool {
        self.0
            .networks
            .iter()
            .any(|n| n.kind == Some(Scope::Mesh) && !n.members.is_empty())
    }
}

impl Diagram for Overlay<'_, '_> {
    fn stem(&self) -> String {
        diagram::OVERLAY.into()
    }

    fn draw(&self, doc: &mut Doc) {
        let v = self.0;
        doc.vertical();
        let meshes = v.networks.iter().filter(|n| n.kind == Some(Scope::Mesh));
        let mut clouds: IndexMap<String, Cloud> = IndexMap::new();
        let mut hosts: IndexSet<&str> = IndexSet::new();
        for n in meshes.filter(|n| !n.members.is_empty()) {
            let mesh = Cloud::Network(n);
            clouds.insert(mesh.key(), mesh);
            for m in &n.members {
                hosts.insert(&m.host);
                doc.line(&[mesh.key()], &[&m.host], Some(&m.interface), mesh.edge());
                for r in &m.routes {
                    let via = Cloud::route(v.networks, r);
                    clouds.insert(via.key(), via);
                    doc.line(
                        &[&m.host],
                        &[via.key()],
                        Some(&t::route(&m.interface)),
                        Class::Route,
                    );
                }
            }
        }
        let mut servers = IndexSet::new();
        for f in v.flows().filter(|f| f.plane == Plane::Control) {
            let (Some(from), Some(to)) = (f.from.host(), f.to.host()) else {
                continue;
            };
            if hosts.contains(from) && from != to {
                servers.insert(to);
                doc.edge(&[from], &[to], Some(f.label), Class::Control);
            }
        }
        for c in clouds.values() {
            c.draw(doc, None);
        }
        for h in hosts.iter().chain(&servers).collect::<IndexSet<_>>() {
            let label = v.label(h);
            let label = if servers.contains(h) {
                t::server(&label)
            } else {
                label
            };
            doc.shape(h, &label, Class::Machine).link(&page::host(h));
        }
    }
}
