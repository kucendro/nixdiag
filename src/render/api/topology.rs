use crate::api::{self, Meta};
use crate::facts::Facts;
use crate::source::annotations::{Endpoint, Model, NodeInfo};

pub(super) fn node_id(e: &Endpoint) -> String {
    match e {
        Endpoint::Host(h) => h.clone(),
        Endpoint::Unit(h, u) => format!("{h}/{u}"),
        Endpoint::Internet => "internet".into(),
        Endpoint::Lan => "lan".into(),
    }
}

fn exposes(model: &Model, host: &str, unit: Option<&str>, info: &NodeInfo) -> Vec<api::Expose> {
    info.exposes
        .iter()
        .map(|e| api::Expose {
            port: e.port,
            protocol: if e.udp { "udp" } else { "tcp" },
            scope: model
                .effective_scope(host, unit, e)
                .map(|s| s.label().to_string()),
            name: e.name.clone(),
        })
        .collect()
}

pub(super) fn build(meta: Meta, facts: &Facts, model: &Model) -> api::Topology {
    let mut nodes: Vec<api::Node> = Vec::new();
    let mut push = |host: &str, unit: Option<&str>, info: &NodeInfo| {
        nodes.push(api::Node {
            id: match unit {
                Some(u) => format!("{host}/{u}"),
                None => host.to_string(),
            },
            host: host.to_string(),
            unit: unit.map(str::to_string),
            role: info.role.clone(),
            scope: info.scope.map(|s| s.label().to_string()),
            exposes: exposes(model, host, unit, info),
            names: info.names.clone(),
        });
    };
    for host in facts.hosts.keys() {
        if let Some(info) = model.hosts.get(host) {
            push(host, None, info);
        }
        for ((h, unit), info) in &model.units {
            if h == host {
                push(host, Some(unit), info);
            }
        }
    }
    nodes.sort_by(|a, b| a.id.cmp(&b.id));

    let mut edges: Vec<api::Edge> = model
        .edges
        .iter()
        .map(|e| api::Edge {
            from: node_id(&e.from),
            to: node_id(&e.to),
            label: (!e.label.is_empty()).then(|| e.label.clone()),
        })
        .collect();
    edges.sort_by(|a, b| {
        a.from
            .cmp(&b.from)
            .then(a.to.cmp(&b.to))
            .then(a.label.cmp(&b.label))
    });

    let mut endpoints: Vec<api::EndpointRow> = Vec::new();
    for n in &nodes {
        for e in &n.exposes {
            endpoints.push(api::EndpointRow {
                name: e
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("{}:{}", n.host, e.port)),
                port: Some(e.port),
                scope: e.scope.clone(),
                node: n.id.clone(),
                target: None,
            });
        }
    }
    for ne in &model.named {
        let (host, unit) = match &ne.node {
            Endpoint::Host(h) => (h.clone(), None),
            Endpoint::Unit(h, u) => (h.clone(), Some(u.clone())),
            _ => continue,
        };
        endpoints.push(api::EndpointRow {
            name: ne.name.clone(),
            port: ne.port,
            scope: model
                .node_scope(&host, unit.as_deref())
                .map(|s| s.label().to_string()),
            node: node_id(&ne.node),
            target: Some(node_id(&ne.target)),
        });
    }
    endpoints.sort_by(|a, b| a.name.cmp(&b.name).then(a.node.cmp(&b.node)));

    api::Topology {
        meta,
        nodes,
        edges,
        endpoints,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_ids_are_spelled_the_way_annotations_write_them() {
        assert_eq!(
            node_id(&Endpoint::Unit("sol".into(), "nginx".into())),
            "sol/nginx"
        );
        assert_eq!(node_id(&Endpoint::Host("nas".into())), "nas");
        assert_eq!(node_id(&Endpoint::Internet), "internet");
        assert_eq!(node_id(&Endpoint::Lan), "lan");
        assert_eq!(
            node_id(&Endpoint::Unit("web-01".into(), "nginx.tls".into())),
            "web-01/nginx.tls"
        );
    }
}
