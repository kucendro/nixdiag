use super::{Net, View};
use crate::conf::files::{diagram, page};
use crate::render::d2::{Class, Diagram, Doc};
use crate::text::d2::networks as t;
use indexmap::IndexSet;
use itertools::Itertools;

pub struct Networks<'v, 'a>(pub &'v View<'a>);

impl Diagram for Networks<'_, '_> {
    fn stem(&self) -> String {
        diagram::NETWORKS.into()
    }

    fn draw(&self, doc: &mut Doc) {
        let v = self.0;
        doc.vertical();
        if v.networks.iter().any(|n| !n.gateways.is_empty()) {
            Net::Internet.draw(doc);
        }
        for n in v.networks {
            let net = Net::of(n.kind).unwrap_or(Net::Lan);
            let key = format!("net {}", n.id());
            let cidrs = n.cidrs.iter().join("\n");
            doc.shape(&key, &t::network(n.name.as_deref(), &cidrs), net.cloud());
            for g in &n.gateways {
                let owner = n.members.iter().find(|m| m.address == Some(*g));
                let gw = match owner {
                    Some(m) => m.host.clone(),
                    None => {
                        let gw = t::gateway(g);
                        doc.shape(&gw, &gw, Class::Ghost);
                        doc.line(&[&gw], &[&key], None, net.edge());
                        gw
                    }
                };
                doc.line(&[Net::Internet.key()], &[&gw], None, Class::Public);
            }
            for m in &n.members {
                let label = t::attach(&m.interface, m.address);
                doc.line(&[&key], &[&m.host], Some(&label), net.edge());
            }
        }
        let hosts: IndexSet<&str> = v
            .networks
            .iter()
            .flat_map(|n| n.members.iter().map(|m| m.host.as_str()))
            .collect();
        for h in hosts {
            doc.shape(h, &v.label(h), Class::Machine)
                .link(&page::host(h));
        }
    }
}
