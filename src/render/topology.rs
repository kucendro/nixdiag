use crate::facts::{Expose, Facts, Host, Kind, Scope};
use crate::render::d2::{preamble, write_and_render, D2Style};
use crate::render::out::Out;
use crate::text::d2::topology as t;
use crate::text::fill;
use crate::topology::{scope_at, Endpoint, Model};
use crate::util::sanitize;
use anyhow::Result;
use indexmap::IndexMap;

fn class(kind: Option<Kind>) -> &'static str {
    match kind {
        Some(Kind::Infra) => "infra",
        _ => "app",
    }
}

fn node_label(unit: &str, role: Option<&str>) -> String {
    match role {
        Some(r) => fill(
            t::UNIT_WITH_ROLE,
            &[("unit", unit), ("role", &r.replace('-', " "))],
        ),
        None => unit.to_string(),
    }
}

fn endpoint_id(e: &Endpoint) -> String {
    match e {
        Endpoint::Host(h) => sanitize(h),
        Endpoint::Unit(h, u) => format!("{}.{}", sanitize(h), sanitize(u)),
        Endpoint::Internet => "internet".into(),
        Endpoint::Lan => "lan".into(),
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

fn connection(from: &Endpoint, to: &Endpoint, label: &str) -> String {
    fill(
        t::CONNECTION,
        &[
            ("from", &endpoint_id(from)),
            ("to", &endpoint_id(to)),
            ("label", &label.replace('"', "'")),
            ("color", color(from, to)),
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

pub fn generate(
    facts: &Facts,
    model: &Model,
    out: &mut Out,
    render_svg: bool,
    style: &D2Style,
) -> Result<()> {
    let mut per_host: IndexMap<&str, IndexMap<&str, (&'static str, String)>> = facts
        .hosts
        .iter()
        .map(|(host, f)| {
            let units = f
                .topology()
                .units
                .iter()
                .map(|(u, info)| {
                    (
                        u.as_str(),
                        (class(info.kind), node_label(u, info.role.as_deref())),
                    )
                })
                .collect();
            (host.as_str(), units)
        })
        .collect();
    for c in &model.connections {
        for ep in [&c.from, &c.to] {
            if let Endpoint::Unit(h, u) = ep {
                if let Some(m) = per_host.get_mut(h.as_str()) {
                    m.entry(u.as_str()).or_insert(("app", u.clone()));
                }
            }
        }
    }

    let mut clouds: Vec<(Endpoint, Endpoint, String)> = Vec::new();
    for (host, f) in &facts.hosts {
        let topo = f.topology();
        let mut collect = |node: Endpoint, unit: Option<&str>, exposes: &[Expose]| {
            for e in exposes {
                if let Some(c) = cloud(e.scope.or_else(|| topo.scope_of(unit))) {
                    clouds.push((
                        c,
                        node.clone(),
                        expose_label(e.name.as_deref(), Some(e.port), e.udp),
                    ));
                }
            }
        };
        collect(Endpoint::Host(host.clone()), None, &topo.expose);
        for (unit, u) in &topo.units {
            collect(
                Endpoint::Unit(host.clone(), unit.clone()),
                Some(unit),
                &u.expose,
            );
        }
    }
    for ne in &model.named {
        if let Some(c) = cloud(ne.scope.or_else(|| scope_at(facts, &ne.node))) {
            clouds.push((
                c,
                ne.node.clone(),
                expose_label(Some(&ne.name), ne.port, false),
            ));
        }
    }

    let used = |cloud: Endpoint| {
        clouds.iter().any(|(c, ..)| *c == cloud)
            || model
                .connections
                .iter()
                .any(|c| c.from == cloud || c.to == cloud)
    };

    let mut o = preamble(style);
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
        for (unit, (class, label)) in per_host.get(host.as_str()).into_iter().flatten() {
            o.push(fill(
                t::UNIT,
                &[
                    ("id", &sanitize(unit)),
                    ("label", &label.replace('"', "'")),
                    ("class", class),
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
    for (c, node, label) in &clouds {
        o.push(connection(c, node, label));
    }
    for c in &model.connections {
        o.push(connection(&c.from, &c.to, &c.label));
    }

    write_and_render(out, "topology", &o, render_svg, style)
}
