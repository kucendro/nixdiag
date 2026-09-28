use super::*;
use serde_json::{json, Value};

fn facts(hosts: Value) -> Facts {
    serde_json::from_value(json!({ "schema": 4, "hosts": hosts })).unwrap()
}

fn host(interfaces: Value, networks: Value) -> Value {
    json!({
        "kind": "nixos",
        "network": {
            "interfaces": interfaces,
            "gateways": [{ "address": "192.168.1.1", "interface": "eth0" }]
        },
        "topology": { "networks": networks }
    })
}

fn eth0(cidr: &str) -> Value {
    json!({ "eth0": { "addresses": [{ "cidr": cidr, "scope": "lan" }] } })
}

fn at(mut host: Value, location: &str) -> Value {
    host["topology"]["location"] = json!(location);
    host
}

fn ids(nets: &[Network]) -> Vec<String> {
    nets.iter().map(Network::id).collect()
}

fn members(n: &Network) -> Vec<(&str, &str, Option<String>)> {
    n.members
        .iter()
        .map(|m| {
            (
                m.host.as_str(),
                m.interface.as_str(),
                m.address.map(|a| a.to_string()),
            )
        })
        .collect()
}

#[test]
fn shared_subnet_is_one_derived_network_with_its_gateway() {
    let f = facts(json!({
        "a": host(eth0("192.168.1.10/24"), json!({})),
        "b": host(eth0("192.168.1.20/24"), json!({})),
    }));
    let nets = build(&f).unwrap();
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0].id(), "192.168.1.0/24");
    assert_eq!(nets[0].kind, Some(Scope::Lan));
    assert_eq!(
        members(&nets[0]),
        [
            ("a", "eth0", Some("192.168.1.10".into())),
            ("b", "eth0", Some("192.168.1.20".into()))
        ]
    );
    assert_eq!(
        nets[0]
            .gateways
            .iter()
            .map(|g| g.to_string())
            .collect::<Vec<_>>(),
        ["192.168.1.1"]
    );
}

#[test]
fn declaration_names_the_subnet_and_host_prefixes_join_only_declared() {
    let lan = json!({ "lan": { "cidrs": ["192.168.1.0/24"] } });
    let wg = json!({ "wg0": { "kind": "wireguard", "addresses": [{ "cidr": "10.9.0.1/32", "scope": "lan" }] } });
    let f = facts(json!({
        "a": host(eth0("192.168.1.10/24"), lan),
        "b": host(wg, json!({})),
    }));
    let nets = build(&f).unwrap();
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0].id(), "lan");
}

#[test]
fn mesh_interface_joins_by_control_server() {
    let tailnet = json!({ "hs.example": { "cidrs": ["100.64.0.0/10"], "kind": "mesh", "server": "hs.example" } });
    let ts = json!({ "tailscale0": { "kind": "mesh", "server": "hs.example" } });
    let f = facts(json!({ "a": host(ts, tailnet) }));
    let nets = build(&f).unwrap();
    assert_eq!(members(&nets[0]), [("a", "tailscale0", None)]);
}

#[test]
fn route_wider_than_a_subnet_is_its_own_network() {
    let mut ifaces = eth0("10.0.0.5/24");
    ifaces["eth0"]["routes"] = json!(["10.0.0.0/8"]);
    let nets = build(&facts(json!({ "a": host(ifaces, json!({})) }))).unwrap();
    let ids: Vec<_> = nets.iter().map(Network::id).collect();
    assert_eq!(ids, ["10.0.0.0/24", "10.0.0.0/8"]);
    let wide = "10.0.0.0/8".parse().unwrap();
    assert_eq!(
        find(&nets, |n| n.covers(&wide), None)
            .unwrap()
            .map(Network::id)
            .unwrap(),
        "10.0.0.0/8"
    );
    let one = "10.0.0.7/32".parse().unwrap();
    assert_eq!(
        find(&nets, |n| n.covers(&one), None)
            .unwrap()
            .map(Network::id)
            .unwrap(),
        "10.0.0.0/24"
    );
}

#[test]
fn conflicting_and_overlapping_declarations_fail() {
    let one = json!({ "lan": { "cidrs": ["192.168.1.0/24"] } });
    let other = json!({ "lan": { "cidrs": ["192.168.2.0/24"] } });
    let f = facts(json!({ "a": host(json!({}), one), "b": host(json!({}), other) }));
    assert!(build(&f)
        .unwrap_err()
        .to_string()
        .contains("declared differently on a and b"));

    let wide =
        json!({ "all": { "cidrs": ["192.168.0.0/16"] }, "iot": { "cidrs": ["192.168.20.0/24"] } });
    let f = facts(json!({ "a": host(json!({}), wide) }));
    assert!(build(&f)
        .unwrap_err()
        .to_string()
        .contains("`all` and `iot` overlap"));
}

#[test]
fn scopeless_addresses_are_skipped_and_bad_ones_fail() {
    let lo = json!({ "lo": { "addresses": [{ "cidr": "127.0.0.1/8" }] } });
    assert!(build(&facts(json!({ "a": host(lo, json!({})) })))
        .unwrap()
        .is_empty());
    let bad = eth0("192.168.1.300/24");
    assert!(build(&facts(json!({ "a": host(bad, json!({})) }))).is_err());
}

#[test]
fn one_subnet_at_two_locations_is_two_networks() {
    let f = facts(json!({
        "a": at(host(eth0("192.168.1.10/24"), json!({})), "home"),
        "b": at(host(eth0("192.168.1.20/24"), json!({})), "lab"),
        "c": at(host(eth0("192.168.2.30/24"), json!({})), "lab"),
    }));
    let nets = build(&f).unwrap();
    assert_eq!(
        ids(&nets),
        [
            "192.168.1.0/24 @ home",
            "192.168.1.0/24 @ lab",
            "192.168.2.0/24 @ lab"
        ]
    );
    assert_eq!(
        members(&nets[1]),
        [("b", "eth0", Some("192.168.1.20".into()))]
    );
    assert_eq!(nets[1].gateways.len(), 1);
}

#[test]
fn an_unlocated_host_joins_the_only_copy_else_fails() {
    let f = facts(json!({
        "c": host(eth0("192.168.1.30/24"), json!({})),
        "a": at(host(eth0("192.168.1.10/24"), json!({})), "home"),
    }));
    let nets = build(&f).unwrap();
    assert_eq!(ids(&nets), ["192.168.1.0/24 @ home"]);
    assert_eq!(nets[0].members.len(), 2);

    let f = facts(json!({
        "a": at(host(eth0("192.168.1.10/24"), json!({})), "home"),
        "b": at(host(eth0("192.168.1.20/24"), json!({})), "lab"),
        "c": host(eth0("192.168.1.30/24"), json!({})),
    }));
    let err = build(&f).unwrap_err().to_string();
    assert!(
        err.contains("c: `192.168.1.30` is on networks at several locations"),
        "{err}"
    );
}

#[test]
fn a_name_splits_per_location_and_a_mesh_never_does() {
    let lan = json!({
        "lan": { "cidrs": ["192.168.1.0/24"] },
        "tailnet": { "cidrs": ["100.64.0.0/10"], "kind": "mesh" }
    });
    let f = facts(json!({
        "a": at(host(eth0("192.168.1.10/24"), lan.clone()), "home"),
        "b": at(host(eth0("192.168.1.20/24"), lan.clone()), "lab"),
    }));
    let nets = build(&f).unwrap();
    assert_eq!(ids(&nets), ["lan @ home", "tailnet", "lan @ lab"]);
    assert_eq!(
        members(&nets[2]),
        [("b", "eth0", Some("192.168.1.20".into()))]
    );

    let f = facts(json!({
        "a": at(host(json!({}), lan.clone()), "home"),
        "b": host(json!({}), lan),
    }));
    let err = build(&f).unwrap_err().to_string();
    assert!(err.contains("`lan @ home` and `lan` overlap"), "{err}");
}

#[test]
fn routes_lead_into_their_own_location() {
    let mut router = eth0("192.168.1.10/24");
    router["eth0"]["routes"] = json!(["192.168.1.0/24", "10.9.0.0/16", "0.0.0.0/0"]);
    let f = facts(json!({
        "a": at(host(router, json!({})), "home"),
        "b": at(host(eth0("192.168.1.20/24"), json!({})), "lab"),
    }));
    let nets = build(&f).unwrap();
    let to: Vec<_> = nets[0].members[0]
        .routes
        .iter()
        .map(|r| r.to.as_deref())
        .collect();
    assert_eq!(
        to,
        [
            Some("192.168.1.0/24 @ home"),
            Some("10.9.0.0/16 @ home"),
            None
        ]
    );
}
