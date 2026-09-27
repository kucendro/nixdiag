use crate::conf::files::diagram;
use crate::facts::{Facts, Host, Kind, Scope};
use crate::render::d2::D2;
use crate::text::d2::topology as t;
use crate::text::fill;
use crate::topology::{Connection, Endpoint, Model, INTERNET, LAN};
use crate::util::sanitize;
use anyhow::Result;
use indexmap::IndexMap;

struct Node {
    class: &'static str,
    label: String,
}

impl Node {
    fn declared(unit: &str, kind: Option<Kind>, role: Option<&str>) -> Node {
        Node {
            class: match kind {
                Some(Kind::Infra) => "infra",
                _ => "app",
            },
            label: match role {
                Some(r) => fill(
                    t::UNIT_WITH_ROLE,
                    &[("unit", unit), ("role", &r.replace('-', " "))],
                ),
                None => unit.to_string(),
            },
        }
    }
}

fn endpoint_id(e: &Endpoint) -> String {
    match e {
        Endpoint::Host(h) => sanitize(h),
        Endpoint::Unit(h, u) => format!("{}.{}", sanitize(h), sanitize(u)),
        Endpoint::Internet => INTERNET.into(),
        Endpoint::Lan => LAN.into(),
    }
}

fn color(a: &Endpoint, b: &Endpoint) -> &'static str {
    if matches!(a, Endpoint::Internet) || matches!(b, Endpoint::Internet) {
        "${public}"
    } else if matches!(a, Endpoint::Lan) || matches!(b, Endpoint::Lan) {
        "${lan}"
    } else {
        "${mesh}"
    }
}

fn connection(c: &Connection) -> String {
    fill(
        t::CONNECTION,
        &[
            ("from", &endpoint_id(&c.from)),
            ("to", &endpoint_id(&c.to)),
            ("label", &c.label.replace('"', "'")),
            ("color", color(&c.from, &c.to)),
        ],
    )
}

fn cloud(scope: Option<Scope>) -> Option<Endpoint> {
    match scope? {
        Scope::Public => Some(Endpoint::Internet),
        Scope::Lan => Some(Endpoint::Lan),
        Scope::Mesh => None,
    }
}

fn expose_label(name: Option<&str>, port: Option<u32>, udp: bool) -> String {
    let port = port.map(|p| p.to_string()).unwrap_or_default();
    let proto = if udp { t::UDP_SUFFIX } else { "" };
    match name {
        Some(n) => fill(
            t::EXPOSE_NAMED,
            &[("name", n), ("port", &port), ("proto", proto)],
        ),
        None => fill(t::EXPOSE, &[("port", &port), ("proto", proto)]),
    }
}

fn fmt_ports(tcp: &[u32], udp: &[u32]) -> String {
    let list = |ps: &[u32]| ps.iter().map(u32::to_string).collect::<Vec<_>>().join(", ");
    let tcp = fill(t::TCP, &[("ports", &list(tcp))]);
    let udp = fill(t::UDP, &[("ports", &list(udp))]);
    match (tcp.is_empty(), udp.is_empty()) {
        (false, false) => fill(t::TCP_AND_UDP, &[("tcp", &tcp), ("udp", &udp)]),
        (false, true) => tcp,
        (true, false) => udp,
        (true, true) => String::new(),
    }
}

pub fn generate(facts: &Facts, model: &Model, d2: &D2) -> Result<()> {
    let mut per_host: IndexMap<&str, IndexMap<&str, Node>> = facts
        .hosts
        .iter()
        .map(|(host, f)| {
            let units = f.topology().units.iter().map(|(u, info)| {
                let node = Node::declared(u, info.kind, info.role.as_deref());
                (u.as_str(), node)
            });
            (host.as_str(), units.collect())
        })
        .collect();
    for c in &model.connections {
        for ep in [&c.from, &c.to] {
            if let Endpoint::Unit(h, u) = ep {
                if let Some(m) = per_host.get_mut(h.as_str()) {
                    m.entry(u.as_str())
                        .or_insert_with(|| Node::declared(u, None, None));
                }
            }
        }
    }

    let exposed = model.exposed.iter().filter_map(|x| {
        Some(Connection {
            from: cloud(x.scope)?,
            to: x.node(),
            label: expose_label(x.name.as_deref(), Some(x.port), x.udp),
        })
    });
    let named = model.named.iter().filter_map(|ne| {
        Some(Connection {
            from: cloud(ne.scope)?,
            to: ne.node.clone(),
            label: expose_label(Some(&ne.name), ne.port, false),
        })
    });
    let edges: Vec<Connection> = exposed.chain(named).collect();
    let edges: Vec<&Connection> = edges.iter().chain(&model.connections).collect();
    let used = |cloud: Endpoint| edges.iter().any(|c| c.from == cloud || c.to == cloud);

    let mut o = d2.preamble();
    o.push(t::CLASSES.into());
    o.push(String::new());
    if used(Endpoint::Internet) {
        o.push(fill(
            t::INTERNET,
            &[("id", &endpoint_id(&Endpoint::Internet))],
        ));
    }
    if used(Endpoint::Lan) {
        o.push(fill(t::LAN, &[("id", &endpoint_id(&Endpoint::Lan))]));
    }
    o.push(String::new());
    for (host, f) in &facts.hosts {
        let icon = match f {
            Host::Darwin(_) => t::DARWIN_ICON,
            Host::Nixos(_) => t::NIXOS_ICON,
        };
        o.push(fill(
            t::HOST_OPEN,
            &[("id", &sanitize(host)), ("icon", icon), ("host", host)],
        ));
        for (unit, node) in per_host.get(host.as_str()).into_iter().flatten() {
            o.push(fill(
                t::UNIT,
                &[
                    ("id", &sanitize(unit)),
                    ("label", &node.label.replace('"', "'")),
                    ("class", node.class),
                ],
            ));
        }
        if facts.bare() {
            if let Some(n) = f.as_nixos() {
                let ports = fmt_ports(&n.tcp, &n.udp);
                if !ports.is_empty() {
                    o.push(fill(t::PORTS, &[("ports", &ports)]));
                }
            }
        }
        o.push(fill(t::BASE, &[("count", &f.svc_count().to_string())]));
        o.push(t::HOST_CLOSE.into());
    }
    o.push(String::new());
    o.push(t::CONNECTIONS.into());
    o.extend(edges.into_iter().map(connection));

    d2.write(diagram::TOPOLOGY, &o)
}
