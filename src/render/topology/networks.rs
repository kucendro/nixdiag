use super::{Cloud, Net, View};
use crate::conf::files::{diagram, page};
use crate::render::d2::{Class, Diagram, Doc};
use crate::text::d2::networks as t;
use indexmap::IndexSet;

pub struct Networks<'v, 'a>(pub &'v View<'a>);

impl Diagram for Networks<'_, '_> {
    fn stem(&self) -> String {
        diagram::NETWORKS.into()
    }

    fn draw(&self, doc: &mut Doc) {
        let v = self.0;
        doc.vertical();
        let internet = Cloud::Net(Net::Internet);
        if v.networks.iter().any(|n| !n.gateways.is_empty()) {
            internet.draw(doc);
        }
        for n in v.networks {
            let cloud = Cloud::Network(n);
            let key = cloud.key();
            cloud.draw(doc);
            for g in &n.gateways {
                let owner = n.members.iter().find(|m| m.address == Some(*g));
                let gw = match owner {
                    Some(m) => m.host.clone(),
                    None => {
                        let gw = t::gateway(g);
                        doc.shape(&gw, &gw, Class::Ghost);
                        doc.line(&[&gw], &[&key], None, cloud.edge());
                        gw
                    }
                };
                doc.line(&[internet.key()], &[&gw], None, internet.edge());
            }
            for m in &n.members {
                let label = t::attach(&m.interface, m.address);
                doc.line(&[&key], &[&m.host], Some(&label), cloud.edge());
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
