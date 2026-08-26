//! Data-flow topology diagram, driven entirely by `#:` annotations.
//! With zero annotations it degrades to host boxes + firewall ports.

use crate::annotations::{Endpoint, Model, Scope};
use crate::d2::{write_and_render, D2_HEADER};
use crate::facts::{Facts, Host};
use crate::output::Out;
use crate::util::sanitize;
use anyhow::Result;
use indexmap::IndexMap;

/// Known roles map to a d2 class and a label suffix; unknown roles render
/// with defaults, so diagrams stay user-programmable without touching nixdiag.
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
        "#c0392b"
    } else if matches!(a, Endpoint::Lan) || matches!(b, Endpoint::Lan) {
        "#27893f"
    } else {
        "#4a76c4"
    }
}

fn fmt_ports(tcp: &[u32], udp: &[u32]) -> String {
    let list = |ps: &[u32]| {
        ps.iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };
    match (tcp.is_empty(), udp.is_empty()) {
        (false, false) => format!("tcp {} · udp {}", list(tcp), list(udp)),
        (false, true) => format!("tcp {}", list(tcp)),
        (true, false) => format!("udp {}", list(udp)),
        (true, true) => String::new(),
    }
}

pub fn generate(facts: &Facts, model: &Model, out: &mut Out, render_svg: bool) -> Result<()> {
    // Nodes to draw per host: every annotated service, plus services that only
    // appear as edge endpoints (any enabled service is a valid target).
    let mut per_host: IndexMap<&str, IndexMap<&str, (&'static str, String)>> = facts
        .hosts
        .keys()
        .map(|h| (h.as_str(), IndexMap::new()))
        .collect();
    for ((host, unit), info) in &model.units {
        let (class, label) = match &info.role {
            Some(r) => {
                let (class, rl) = role_style(r);
                (class, format!("{unit}\\n({rl})"))
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

    // Expose edges: public ones come in from the internet, lan ones from the
    // LAN cloud; mesh endpoints are only listed on the endpoints page.
    let mut expose_edges: Vec<(Endpoint, Endpoint, String)> = Vec::new();
    let mut collect =
        |node: Endpoint, host: &str, unit: Option<&str>, info: &crate::annotations::NodeInfo| {
            for e in &info.exposes {
                let cloud = match model.effective_scope(host, unit, e) {
                    Some(Scope::Public) => Endpoint::Internet,
                    Some(Scope::Lan) => Endpoint::Lan,
                    _ => continue,
                };
                let proto = if e.udp { "/udp" } else { "" };
                let label = match &e.name {
                    Some(n) => format!("{n} :{}{proto}", e.port),
                    None => format!(":{}{proto}", e.port),
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

    let internet_used = expose_edges.iter().any(|(c, ..)| *c == Endpoint::Internet)
        || model
            .edges
            .iter()
            .any(|e| e.from == Endpoint::Internet || e.to == Endpoint::Internet);
    let lan_used = expose_edges.iter().any(|(c, ..)| *c == Endpoint::Lan)
        || model
            .edges
            .iter()
            .any(|e| e.from == Endpoint::Lan || e.to == Endpoint::Lan);

    // --- emit ------------------------------------------------------------
    let mut o: Vec<String> = D2_HEADER.iter().map(|s| s.to_string()).collect();
    o.extend(
        [
            "direction: right",
            "classes: {",
            "  app: { style: { fill: \"#e6f0ff\"; stroke: \"#4a76c4\" } }",
            "  infra: { style: { fill: \"#ffe9cc\"; stroke: \"#c47a29\" } }",
            "  base: { style: { fill: \"#f0f0f0\"; stroke: \"#999\"; font-size: 13 } }",
            "}",
            "",
        ]
        .map(String::from),
    );
    if internet_used {
        o.push("internet: \"🌐 Internet\" { shape: cloud; style.fill: \"#fdecea\" }".into());
    }
    if lan_used {
        o.push("lan: \"🏠 LAN\" { shape: cloud; style.fill: \"#eafaf1\" }".into());
    }
    o.push(String::new());
    for (host, f) in &facts.hosts {
        let icon = match f {
            Host::Darwin(_) => "🍏",
            Host::Nixos(_) => "🖥️",
        };
        o.push(format!("{}: \"{icon} {host}\" {{", sanitize(host)));
        o.push("  style: { fill: \"#fbfbfe\"; stroke: \"#333\"; bold: true }".into());
        for (unit, (class, label)) in per_host.get(host.as_str()).into_iter().flatten() {
            let safe = label.replace('"', "'");
            o.push(format!(
                "  {}: \"{safe}\" {{ class: {class} }}",
                sanitize(unit)
            ));
        }
        if model.total == 0 {
            if let Some(n) = f.as_nixos() {
                let ports = fmt_ports(&n.tcp, &n.udp);
                if !ports.is_empty() {
                    o.push(format!("  ports: \"{ports}\" {{ class: base }}"));
                }
            }
        }
        o.push(format!(
            "  base: \"+ {} system services\" {{ class: base }}",
            f.svc_count()
        ));
        o.push("}".into());
    }
    o.push(String::new());
    o.push("# data-flow edges".into());
    for (cloud, node, label) in &expose_edges {
        let lbl = label.replace('"', "'");
        o.push(format!(
            "{} -> {}: \"{lbl}\" {{ style.stroke: \"{}\" }}",
            endpoint_id(cloud),
            endpoint_id(node),
            edge_color(cloud, node),
        ));
    }
    for e in &model.edges {
        let lbl = e.label.replace('"', "'");
        o.push(format!(
            "{} -> {}: \"{lbl}\" {{ style.stroke: \"{}\" }}",
            endpoint_id(&e.from),
            endpoint_id(&e.to),
            edge_color(&e.from, &e.to),
        ));
    }

    write_and_render(out, "topology", &o, render_svg)
}
