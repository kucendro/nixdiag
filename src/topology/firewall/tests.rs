use super::Finding::*;
use super::*;
use crate::topology::build;
use serde_json::{json, Value};

fn eth0(cidr: &str) -> Value {
    json!({ "eth0": { "addresses": [{ "cidr": cidr, "scope": "lan" }] } })
}

fn host(firewall: Value, units: Value) -> Value {
    json!({
        "kind": "nixos",
        "network": { "interfaces": eth0("192.168.1.20/24"), "firewall": firewall },
        "topology": { "units": units }
    })
}

fn findings(server: Value) -> Vec<Finding> {
    let client = json!({
        "kind": "nixos",
        "network": { "interfaces": eth0("192.168.1.10/24"), "firewall": { "enable": true } },
        "topology": { "units": { "proxy": { "connections": [{ "to": "http://b:3000" }] } } }
    });
    let facts: Facts =
        serde_json::from_value(json!({ "schema": 4, "hosts": { "a": client, "b": server } }))
            .unwrap();
    build(&facts).unwrap().findings.swap_remove("b").unwrap()
}

fn web(scope: &str) -> Value {
    json!({ "web": { "ports": [3000], "expose": [{ "port": 3000, "scope": scope }] } })
}

#[test]
fn open_ports_nothing_uses_are_reported_per_interface() {
    let fw =
        json!({ "enable": true, "tcp": [22, 3000], "interfaces": { "eth0": { "udp": [53] } } });
    assert_eq!(
        findings(host(fw, web("lan"))),
        [
            Unused {
                port: 22,
                udp: false,
                on: None
            },
            Unused {
                port: 53,
                udp: true,
                on: Some("eth0".into())
            }
        ]
    );
}

#[test]
fn a_closed_expose_and_a_blocked_path_are_reported() {
    let fw = json!({ "enable": true, "trusted": ["lo", "wg0"] });
    assert_eq!(
        findings(host(fw, web("lan"))),
        [
            Closed {
                unit: Some("web".into()),
                port: 3000,
                udp: false,
                scope: Scope::Lan,
                on: vec!["eth0".into()]
            },
            Trusted("wg0".into()),
            Blocked {
                from: "a/proxy".into(),
                port: 3000,
                on: vec!["eth0".into()]
            }
        ]
    );
}

#[test]
fn ranges_interface_rules_and_a_disabled_firewall_leave_nothing_to_report() {
    for fw in [
        json!({ "enable": true, "tcpRanges": [[2000, 4000]] }),
        json!({ "enable": true, "interfaces": { "eth0": { "tcp": [3000] } } }),
        json!({ "enable": false, "tcp": [22], "trusted": ["wg0"] }),
    ] {
        assert_eq!(findings(host(fw, web("lan"))), []);
    }
}

#[test]
fn a_scopeless_expose_is_reached_from_the_networks_whose_interface_opens_it() {
    let reached = |fw: Value| {
        let facts: Facts = serde_json::from_value(json!({
            "schema": 4,
            "hosts": { "b": host(fw, json!({ "web": { "expose": [{ "port": 3000 }] } })) }
        }))
        .unwrap();
        let x = build(&facts).unwrap().exposed.remove(0);
        (x.scope, x.via)
    };
    let open = json!({ "enable": true, "interfaces": { "eth0": { "tcp": [3000] } } });
    assert_eq!(
        reached(open),
        (Some(Scope::Lan), vec!["192.168.1.0/24".to_string()])
    );
    assert_eq!(reached(json!({ "enable": true })), (None, vec![]));
}
