use crate::facts::{Facts, Host};
use crate::render::d2::{preamble, write_and_render, D2Style};
use crate::render::out::Out;
use crate::source::annotations::{Endpoint, Model, NodeInfo, Scope};
use crate::text::d2::topology as t;
use crate::text::fill;
use crate::util::sanitize;
use anyhow::Result;
use indexmap::IndexMap;

fn role_style(role: &str) -> (&'static str, String) {
    let class = match role {
        "mesh-control" | "proxy" | "monitor" | "dns" | "storage" | "gateway" => "infra",
        _ => "app",
    };
    (class, role.replace('-', " "))
}

fn endpoint_id(e: &Endpoint) -> String {
    match e {
        Endpoint::Host(h) => sanitize(h),
        Endpoint::Unit(h, u) => format!("{}.{}", sanitize(h), sanitize(u)),
        Endpoint::Internet => "internet".into(),
        Endpoint::Lan => "lan".into(),
    }
}

fn edge_color(a: &Endpoint, b: &Endpoint) -> &'static str {
    if matches!(a, Endpoint::Internet) || matches!(b, Endpoint::Internet) {
        "${public}"
    } else if matches!(a, Endpoint::Lan) || matches!(b, Endpoint::Lan) {
        "${lan}"
    } else {
        "${mesh}"
    }
}

fn edge(from: &Endpoint, to: &Endpoint, label: &str) -> String {
    fill(
        t::EDGE,
        &[
            ("from", &endpoint_id(from)),
            ("to", &endpoint_id(to)),
            ("label", &label.replace('"', "'")),
            ("color", edge_color(from, to)),
        ],
    )
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
        .keys()
        .map(|h| (h.as_str(), IndexMap::new()))
        .collect();
    for ((host, unit), info) in &model.units {
        let (class, label) = match &info.role {
            Some(r) => {
                let (class, role) = role_style(r);
                (
                    class,
                    fill(t::UNIT_WITH_ROLE, &[("unit", unit), ("role", &role)]),
                )
            }
            None => ("app", unit.clone()),
        };
        if let Some(m) = per_host.get_mut(host.as_str()) {
            m.insert(unit.as_str(), (class, label));
        }
    }
    for e in &model.edges {
        for ep in [&e.from, &e.to] {
            if let Endpoint::Unit(h, u) = ep {
                if let Some(m) = per_host.get_mut(h.as_str()) {
                    m.entry(u.as_str()).or_insert(("app", u.clone()));
                }
            }
        }
    }

    let mut expose_edges: Vec<(Endpoint, Endpoint, String)> = Vec::new();
    let mut collect = |node: Endpoint, host: &str, unit: Option<&str>, info: &NodeInfo| {
        for e in &info.exposes {
            let cloud = match model.effective_scope(host, unit, e) {
                Some(Scope::Public) => Endpoint::Internet,
                Some(Scope::Lan) => Endpoint::Lan,
                _ => continue,
            };
            let proto = if e.udp { t::UDP_SUFFIX } else { "" };
            let port = e.port.to_string();
            let label = match &e.name {
                Some(n) => fill(
                    t::EXPOSE_NAMED,
                    &[("name", n), ("port", &port), ("proto", proto)],
                ),
                None => fill(t::EXPOSE, &[("port", &port), ("proto", proto)]),
            };
            expose_edges.push((cloud, node.clone(), label));
        }
    };
    for (host, info) in &model.hosts {
        collect(Endpoint::Host(host.clone()), host, None, info);
    }
    for ((host, unit), info) in &model.units {
        collect(
            Endpoint::Unit(host.clone(), unit.clone()),
            host,
            Some(unit),
            info,
        );
    }

    let used = |cloud: Endpoint| {
        expose_edges.iter().any(|(c, ..)| *c == cloud)
            || model.edges.iter().any(|e| e.from == cloud || e.to == cloud)
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
        if model.total == 0 {
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
    o.push(t::EDGES.into());
    for (cloud, node, label) in &expose_edges {
        o.push(edge(cloud, node, label));
    }
    for e in &model.edges {
        o.push(edge(&e.from, &e.to, &e.label));
    }

    write_and_render(out, "topology", &o, render_svg, style)
}
