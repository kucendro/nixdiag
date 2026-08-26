//! Data-flow topology diagram — port of gen-topology.py.

use crate::d2::{write_and_render, D2_HEADER};
use crate::facts::{Facts, Host};
use crate::output::Out;
use crate::repo::{rel_from_store, Repo};
use crate::util::{resolve_upstream, sanitize, split_host_port};
use anyhow::Result;
use indexmap::IndexMap;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// headscale base_domain names + policy.json host CIDRs -> host name.
fn address_book(facts: &Facts, repo: &Repo) -> HashMap<String, String> {
    let mut base = String::new();
    let mut policy_path = String::new();
    for f in facts.hosts.values().filter_map(Host::as_nixos) {
        if f.headscale {
            if !f.base_domain.is_empty() {
                base = f.base_domain.clone();
            }
            if !f.policy_path.is_empty() {
                policy_path = f.policy_path.clone();
            }
        }
    }
    let mut book = HashMap::new();
    if !base.is_empty() {
        for h in facts.hosts.keys() {
            book.insert(format!("{h}.{base}"), h.clone());
        }
    }
    if !policy_path.is_empty() {
        // The eval-time path may not exist here (mode B sandbox); fall back to
        // resolving it inside the repo source.
        let candidates = [
            Path::new(&policy_path).to_path_buf(),
            rel_from_store(&policy_path)
                .map(|r| repo.root.join(r))
                .unwrap_or_default(),
        ];
        for p in candidates {
            let Ok(text) = std::fs::read_to_string(&p) else {
                continue;
            };
            if let Ok(serde_json::Value::Object(data)) = serde_json::from_str(&text) {
                if let Some(serde_json::Value::Object(hosts)) = data.get("hosts") {
                    for (name, cidr) in hosts {
                        if let serde_json::Value::String(cidr) = cidr {
                            let ip = cidr.split('/').next().unwrap_or(cidr);
                            book.insert(ip.to_string(), name.clone());
                        }
                    }
                }
            }
            break;
        }
    }
    book
}

struct Nodes {
    present: HashSet<(String, String)>,
    per_host: IndexMap<String, Vec<String>>,
}

impl Nodes {
    fn node(&mut self, host: &str, nid: &str, label: &str, cls: &str) -> String {
        let sid = sanitize(nid);
        let refid = format!("{}.{}", sanitize(host), sid);
        if self.present.insert((host.to_string(), sid.clone())) {
            let safe = label.replace('"', "'").replace('\n', "\\n");
            self.per_host
                .get_mut(host)
                .unwrap()
                .push(format!("  {sid}: \"{safe}\" {{ class: {cls} }}"));
        }
        refid
    }
}

pub fn generate(facts: &Facts, repo: &Repo, out: &mut Out, render_svg: bool) -> Result<()> {
    let book = address_book(facts, repo);

    let mut vhost_host: HashMap<String, String> = HashMap::new();
    for (h, f) in &facts.hosts {
        if let Some(n) = f.as_nixos() {
            for vh in &n.vhosts {
                vhost_host.insert(vh.name.clone(), h.clone());
            }
        }
    }

    let control_host: Option<String> = facts
        .hosts
        .iter()
        .find(|(_, f)| f.as_nixos().is_some_and(|n| n.headscale))
        .map(|(h, _)| h.clone());
    let hub_host: Option<String> = facts
        .hosts
        .iter()
        .find(|(_, f)| f.as_nixos().is_some_and(|n| n.beszel_hub))
        .map(|(h, _)| h.clone());

    // "host:port" -> (known host name or None, port); loopback means local.
    let resolve = |hostport: &str, local: &str| -> (Option<String>, String) {
        let (host, port) = split_host_port(hostport);
        if matches!(host, "127.0.0.1" | "localhost" | "::1" | "") {
            return (Some(local.to_string()), port.to_string());
        }
        if let Some(h) = book.get(host) {
            return (Some(h.clone()), port.to_string());
        }
        if let Some(h) = vhost_host.get(host) {
            return (Some(h.clone()), port.to_string());
        }
        (None, port.to_string())
    };

    let mut nodes = Nodes {
        present: HashSet::new(),
        per_host: facts
            .hosts
            .keys()
            .map(|h| (h.clone(), Vec::new()))
            .collect(),
    };
    let mut edges: Vec<(String, String, String, &str)> = Vec::new();
    let mut internet_used = false;
    let mut lan_used = false;

    for (host, f) in &facts.hosts {
        let Some(n) = f.as_nixos() else { continue };
        if n.headscale {
            nodes.node(host, "headscale", "headscale\n(mesh control)", "infra");
        }
        if n.vhosts
            .iter()
            .any(|v| resolve_upstream(v.pass.as_deref(), &v.extra).is_some())
        {
            nodes.node(host, "nginx", "nginx\n(reverse proxy)", "infra");
        }
        if n.beszel_hub {
            nodes.node(host, "beszel_hub", "beszel hub", "infra");
        }
        if n.prometheus {
            nodes.node(host, "prometheus", "prometheus", "infra");
        }
        if n.blackbox {
            nodes.node(host, "blackbox", "blackbox exporter", "infra");
        }
        if n.grafana {
            nodes.node(host, "grafana", "grafana", "infra");
        }
        if n.routes.iter().any(|fl| fl.contains("--advertise-routes=")) {
            nodes.node(host, "subnet_router", "subnet router", "infra");
        }
    }

    for (host, f) in &facts.hosts {
        let Some(n) = f.as_nixos() else { continue };
        let mut local_infra: HashMap<String, &str> = HashMap::new();
        if n.headscale {
            local_infra.insert(n.headscale_port.to_string(), "headscale");
        }
        if n.beszel_hub {
            local_infra.insert(n.beszel_hub_port.to_string(), "beszel_hub");
        }
        for vh in &n.vhosts {
            let Some(up) = resolve_upstream(vh.pass.as_deref(), &vh.extra) else {
                continue;
            };
            let (thost, port) = resolve(&up, host);
            let sub = vh.name.split('.').next().unwrap_or(&vh.name).to_string();

            let public = vh.listen.is_empty();
            let app = if thost.as_deref() == Some(host) && local_infra.contains_key(&port) {
                format!("{}.{}", sanitize(host), local_infra[&port])
            } else if thost.as_ref().is_some_and(|t| facts.hosts.contains_key(t)) {
                nodes.node(thost.as_ref().unwrap(), &sub, &sub, "app")
            } else {
                nodes.node(host, &format!("ext_{sub}"), &up, "app")
            };
            edges.push((
                format!("{}.nginx", sanitize(host)),
                app,
                format!("{sub} :{port}"),
                "#4a76c4",
            ));
            if public {
                internet_used = true;
                edges.push((
                    "internet".into(),
                    format!("{}.nginx", sanitize(host)),
                    sub,
                    "#c0392b",
                ));
            }
        }
    }

    for (host, f) in &facts.hosts {
        let member = match f {
            Host::Nixos(n) => n.tailscale,
            Host::Darwin(d) => d.casks.iter().any(|c| c.contains("tailscale")),
        };
        if let Some(control) = &control_host {
            if member && host != control {
                edges.push((
                    sanitize(host),
                    format!("{}.headscale", sanitize(control)),
                    "tailnet".into(),
                    "#7a4fb5",
                ));
            }
        }
    }

    let routes_re = Regex::new(r"--advertise-routes=(\S+)").unwrap();
    for (host, f) in &facts.hosts {
        let Some(n) = f.as_nixos() else { continue };
        for fl in &n.routes {
            if let Some(m) = routes_re.captures(fl) {
                lan_used = true;
                for net in m[1].split(',') {
                    edges.push((
                        format!("{}.subnet_router", sanitize(host)),
                        "lan".into(),
                        format!("advertise {net}"),
                        "#27893f",
                    ));
                }
            }
        }
    }

    for (host, f) in &facts.hosts {
        let agent = match f {
            Host::Nixos(n) => n.beszel_agent,
            Host::Darwin(d) => d.daemons.iter().any(|x| x.contains("beszel")),
        };
        if let Some(hub) = &hub_host {
            if agent && host != hub {
                edges.push((
                    sanitize(host),
                    format!("{}.beszel_hub", sanitize(hub)),
                    "metrics".into(),
                    "#888",
                ));
            }
        }
    }

    let scheme_re = Regex::new(r"^https?://").unwrap();
    for (host, f) in &facts.hosts {
        let Some(n) = f.as_nixos() else { continue };
        if !n.prometheus {
            continue;
        }
        let mut seen: HashSet<String> = HashSet::new();
        for t in &n.prom_targets {
            let hp = scheme_re.replace(t, "").trim_end_matches('/').to_string();
            let (th, _) = resolve(&hp, host);
            if let Some(th) = th {
                if th != *host && seen.insert(th.clone()) {
                    edges.push((
                        format!("{}.prometheus", sanitize(host)),
                        sanitize(&th),
                        "scrape / probe".into(),
                        "#888",
                    ));
                }
            }
        }
    }

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
        let mut block = nodes
            .per_host
            .shift_remove(host.as_str())
            .unwrap_or_default();
        block.push(format!(
            "  base: \"+ {} system services\" {{ class: base }}",
            f.svc_count()
        ));
        o.push(format!("{}: \"{icon} {host}\" {{", sanitize(host)));
        o.push("  style: { fill: \"#fbfbfe\"; stroke: \"#333\"; bold: true }".into());
        o.extend(block);
        o.push("}".into());
    }
    o.push(String::new());
    o.push("# data-flow edges".into());
    for (a, b, label, color) in &edges {
        let lbl = label.replace('"', "'");
        o.push(format!(
            "{a} -> {b}: \"{lbl}\" {{ style.stroke: \"{color}\" }}"
        ));
    }

    write_and_render(out, "topology", &o, render_svg)
}
