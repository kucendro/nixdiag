use super::dot::{id, Diagram, Dot, Graph, Paint};
use crate::conf::files::diagram;
use crate::conf::palette::{diagram as p, Color};
use crate::facts::{Facts, Host, Kind, Scope};
use crate::text::dot::topology as t;
use crate::topology::{Connection, Endpoint, Model, INTERNET, LAN};
use dot_writer::Attributes;
use indexmap::IndexMap;
use std::iter::once;

struct Unit {
    infra: bool,
    role: Option<String>,
}

impl Unit {
    fn declared(kind: Option<Kind>, role: Option<&str>) -> Unit {
        Unit {
            infra: matches!(kind, Some(Kind::Infra)),
            role: role.map(t::role),
        }
    }

    fn colors(&self) -> (Color, Color) {
        match self.infra {
            true => (p::INFRA_FILL, p::INFRA_STROKE),
            false => (p::APP_FILL, p::APP_STROKE),
        }
    }
}

fn base(host: &str) -> String {
    id(&format!("{host}/+base"))
}

fn node(e: &Endpoint) -> String {
    match e {
        Endpoint::Host(h) => base(h),
        Endpoint::Unit(h, u) => id(&format!("{h}/{u}")),
        Endpoint::Internet => id(INTERNET),
        Endpoint::Lan => id(LAN),
    }
}

fn scope_color(c: &Connection) -> Color {
    let touches = |e: Endpoint| c.from == e || c.to == e;
    if touches(Endpoint::Internet) {
        p::PUBLIC
    } else if touches(Endpoint::Lan) {
        p::LAN
    } else {
        p::MESH
    }
}

fn cloud(scope: Option<Scope>) -> Option<Endpoint> {
    match scope? {
        Scope::Public => Some(Endpoint::Internet),
        Scope::Lan => Some(Endpoint::Lan),
        Scope::Mesh => None,
    }
}

fn ports(tcp: &[u32], udp: &[u32]) -> Option<String> {
    let list = |ps: &[u32]| ps.iter().map(u32::to_string).collect::<Vec<_>>().join(", ");
    let mut parts = Vec::new();
    if !tcp.is_empty() {
        parts.push(t::tcp(&list(tcp)));
    }
    if !udp.is_empty() {
        parts.push(t::udp(&list(udp)));
    }
    (!parts.is_empty()).then(|| parts.join(t::PORTS_SEP))
}

pub struct Topology<'a> {
    facts: &'a Facts,
    model: &'a Model,
    per_host: IndexMap<&'a str, IndexMap<&'a str, Unit>>,
    clouds: Vec<Connection>,
}

impl<'a> Topology<'a> {
    pub fn new(facts: &'a Facts, model: &'a Model) -> Self {
        let exposed = model.exposed.iter().filter_map(|x| {
            Some(Connection {
                from: cloud(x.scope)?,
                to: x.node(),
                label: t::expose(x.name.as_deref(), Some(x.port), x.udp),
            })
        });
        let named = model.named.iter().filter_map(|ne| {
            Some(Connection {
                from: cloud(ne.scope)?,
                to: ne.node.clone(),
                label: t::expose(Some(&ne.name), ne.port, false),
            })
        });
        Topology {
            facts,
            model,
            per_host: per_host(facts, model),
            clouds: exposed.chain(named).collect(),
        }
    }

    fn edges(&self) -> impl Iterator<Item = &Connection> {
        self.clouds.iter().chain(&self.model.connections)
    }
}

fn per_host<'a>(facts: &'a Facts, model: &'a Model) -> IndexMap<&'a str, IndexMap<&'a str, Unit>> {
    let mut per_host: IndexMap<&str, IndexMap<&str, Unit>> = facts
        .hosts
        .iter()
        .map(|(host, f)| {
            let units = f
                .topology()
                .units
                .iter()
                .map(|(u, info)| (u.as_str(), Unit::declared(info.kind, info.role.as_deref())));
            (host.as_str(), units.collect())
        })
        .collect();
    for c in &model.connections {
        for ep in [&c.from, &c.to] {
            if let Endpoint::Unit(h, u) = ep {
                if let Some(m) = per_host.get_mut(h.as_str()) {
                    m.entry(u.as_str())
                        .or_insert_with(|| Unit::declared(None, None));
                }
            }
        }
    }
    per_host
}

impl Diagram for Topology<'_> {
    fn stem(&self) -> &'static str {
        diagram::TOPOLOGY
    }

    fn draw(&self, g: &mut Graph, dot: &Dot) {
        let used = |cloud: &Endpoint| self.edges().any(|c| &c.from == cloud || &c.to == cloud);
        let clouds = [
            (Endpoint::Internet, t::INTERNET, p::PUBLIC),
            (Endpoint::Lan, t::LAN, p::LAN),
        ];
        for (cloud, label, stroke) in clouds.iter().filter(|(c, _, _)| used(c)) {
            g.node_named(node(cloud))
                .text(&[label])
                .shape("ellipse")
                .fill(dot.color(&p::HOST_FILL))
                .stroke(dot.color(stroke));
        }
        for (host, f) in &self.facts.hosts {
            let icon = match f {
                Host::Darwin(_) => t::DARWIN_ICON,
                Host::Nixos(_) => t::NIXOS_ICON,
            };
            let mut c = g.cluster();
            c.bold(&t::host(icon, host))
                .fill(dot.color(&p::HOST_FILL))
                .stroke(dot.color(&p::HOST_STROKE))
                .set("style", "rounded,filled", true);
            for (unit, u) in self.per_host.get(host.as_str()).into_iter().flatten() {
                let (fill, stroke) = u.colors();
                let lines: Vec<&str> = once(*unit).chain(u.role.as_deref()).collect();
                c.node_named(id(&format!("{host}/{unit}")))
                    .text(&lines)
                    .fill(dot.color(&fill))
                    .stroke(dot.color(&stroke));
            }
            let open = f.as_nixos().and_then(|n| ports(&n.tcp, &n.udp));
            if let Some(open) = open.filter(|_| self.facts.bare()) {
                c.node_named(id(&format!("{host}/+ports")))
                    .text(&[&open])
                    .set_font_size(11.0);
            }
            c.node_named(base(host))
                .text(&[&t::base(f.svc_count())])
                .set_font_size(11.0);
        }
        for e in self.edges() {
            g.edge(node(&e.from), node(&e.to))
                .attributes()
                .text(&[&e.label])
                .stroke(dot.color(&scope_color(e)));
        }
    }
}
