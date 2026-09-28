use super::{Cloud, Net, Target, View};
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
        let hosts: IndexSet<&str> = v
            .networks
            .iter()
            .flat_map(|n| n.members.iter().map(|m| m.host.as_str()))
            .collect();
        v.boxes(doc, hosts.iter().copied());
        let internet = Cloud::Net(Net::Internet);
        let routes = v
            .networks
            .iter()
            .flat_map(|n| &n.members)
            .flat_map(|m| m.routes.iter().map(move |r| (m, r)));
        let exits = routes.clone().any(|(_, r)| r.to.is_none());
        if exits || v.networks.iter().any(|n| !n.gateways.is_empty()) {
            internet.draw(doc, None);
        }
        for (m, r) in routes {
            let via = Target::Net(Cloud::route(v.networks, r));
            let host = v.placed(&Target::Node(&m.host, None));
            doc.line(
                &host,
                &v.placed(&via),
                Some(&t::route(&m.interface)),
                Class::Route,
            );
        }
        for n in v.networks {
            let cloud = Cloud::Network(n);
            let home = v.home(&Target::Net(cloud));
            let net = v.placed(&Target::Net(cloud));
            cloud.draw(doc, home.as_deref());
            for g in &n.gateways {
                let owner = n.members.iter().find(|m| m.address == Some(*g));
                let gw = match owner {
                    Some(m) => v.placed(&Target::Node(&m.host, None)),
                    None => {
                        let gw = t::gateway(g);
                        doc.place(home.as_deref(), &gw, &gw, Class::Ghost);
                        let path: Vec<String> = home.iter().cloned().chain([gw]).collect();
                        doc.line(&path, &net, None, cloud.edge());
                        path
                    }
                };
                doc.line(&[internet.key()], &gw, None, internet.edge());
            }
            for m in &n.members {
                let label = t::attach(&m.interface, m.address);
                let host = v.placed(&Target::Node(&m.host, None));
                doc.line(&net, &host, Some(&label), cloud.edge());
            }
        }
        for h in hosts {
            let home = v.home(&Target::Node(h, None));
            doc.place(home.as_deref(), h, &v.label(h), Class::Machine)
                .link(&page::host(h));
        }
    }
}
