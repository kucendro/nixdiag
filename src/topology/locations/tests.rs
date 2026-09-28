use super::*;
use crate::topology::networks;
use serde_json::{json, Value};

fn host(cidr: &str, scope: &str, location: Option<&str>) -> Value {
    json!({
        "kind": "nixos",
        "network": { "interfaces": { "eth0": { "addresses": [{ "cidr": cidr, "scope": scope }] } } },
        "topology": { "location": location }
    })
}

fn locations(hosts: Value) -> Vec<(String, String)> {
    let facts: Facts = serde_json::from_value(json!({ "schema": 4, "hosts": hosts })).unwrap();
    let nets = networks::build(&facts).unwrap();
    build(&facts, &nets).into_iter().collect()
}

fn pair(host: &str, location: &str) -> (String, String) {
    (host.into(), location.into())
}

#[test]
fn a_named_host_names_its_subnet() {
    let got = locations(json!({
        "a": host("192.168.1.10/24", "lan", Some("home")),
        "b": host("192.168.1.20/24", "lan", None),
        "c": host("192.168.2.30/24", "lan", None),
    }));
    assert_eq!(got, [pair("a", "home"), pair("b", "home")]);
}

#[test]
fn public_subnets_group_and_mesh_does_not() {
    let got = locations(json!({
        "a": host("203.0.113.10/24", "public", Some("dc")),
        "b": host("203.0.113.20/24", "public", None),
        "c": host("100.64.0.3/10", "mesh", None),
        "d": host("100.64.0.4/10", "mesh", Some("office")),
    }));
    assert_eq!(got, [pair("a", "dc"), pair("b", "dc"), pair("d", "office")]);
}

#[test]
fn conflicting_names_leave_the_unnamed_out() {
    let got = locations(json!({
        "a": host("192.168.1.10/24", "lan", Some("home")),
        "b": host("192.168.1.20/24", "lan", Some("lab")),
        "c": host("192.168.1.30/24", "lan", None),
    }));
    assert_eq!(got, [pair("a", "home"), pair("b", "lab")]);
}
