use super::*;
use serde_json::{json, Value};

fn host(interfaces: Value, units: Value, networks: Value) -> Value {
    json!({
        "kind": "nixos",
        "network": { "interfaces": interfaces },
        "topology": { "units": units, "networks": networks }
    })
}

fn eth0(cidr: &str) -> Value {
    json!({ "eth0": { "addresses": [{ "cidr": cidr, "scope": "lan" }] } })
}

fn to(target: &str, networks: Value) -> Result<Endpoint> {
    let client = json!({ "app": { "connections": [{ "to": target }] } });
    let server = json!({ "web": { "ports": [3000] } });
    let facts: Facts = serde_json::from_value(json!({
        "schema": 4,
        "hosts": {
            "a": host(eth0("192.168.1.10/24"), client, networks),
            "b": host(eth0("192.168.1.20/24"), server, json!({}))
        }
    }))
    .unwrap();
    let model = build(&facts)?;
    Ok(model.connections[0].to.clone())
}

fn net(id: &str) -> Endpoint {
    Endpoint::Network(id.into())
}

#[test]
fn networks_resolve_by_name_and_cidr() {
    let lan = json!({ "lan": { "cidrs": ["192.168.1.0/24"] } });
    assert_eq!(to("lan", lan.clone()).unwrap(), net("lan"));
    assert_eq!(to("192.168.1.0/24", lan).unwrap(), net("lan"));
    assert_eq!(
        to("192.168.1.0/24", json!({})).unwrap(),
        net("192.168.1.0/24")
    );
    assert_eq!(to("10.20.0.0/16", json!({})).unwrap(), net("10.20.0.0/16"));
    assert_eq!(
        to("192.168.0.0/16", json!({})).unwrap(),
        net("192.168.0.0/16")
    );
}

#[test]
fn ip_resolves_to_owner_unit_else_its_network() {
    let unit = Endpoint::Unit("b".into(), "web".into());
    assert_eq!(to("http://192.168.1.20:3000", json!({})).unwrap(), unit);
    assert_eq!(
        to("http://192.168.1.20:22", json!({})).unwrap(),
        Endpoint::Host("b".into())
    );
    assert_eq!(
        to("http://192.168.1.50", json!({})).unwrap(),
        net("192.168.1.0/24")
    );
    assert!(to("http://10.0.0.1", json!({})).is_err());
}

#[test]
fn hosts_win_over_networks_and_bare_lan_says_what_to_do() {
    let clash = json!({ "b": { "cidrs": ["10.1.0.0/16"] } });
    assert_eq!(to("b", clash).unwrap(), Endpoint::Host("b".into()));
    let err = to("lan", json!({})).unwrap_err().to_string();
    assert!(err.contains("a/app -> lan"), "{err}");
    let err = format!("{:#}", to("lan", json!({})).unwrap_err());
    assert!(err.contains("declare nixdiag.networks.lan.cidrs"), "{err}");
}

#[test]
fn addresses_and_names_resolve_at_the_source_location_first() {
    let web = json!({ "web": { "ports": [3000] } });
    let lan = json!({ "lan": { "cidrs": ["192.168.1.0/24"] } });
    let placed = |cidr: &str, units: Value, location: Option<&str>| {
        let networks = if location.is_some() {
            lan.clone()
        } else {
            json!({})
        };
        let mut h = host(eth0(cidr), units, networks);
        h["topology"]["location"] = json!(location);
        h
    };
    let resolve = |target: &str, location: Option<&str>| {
        let client = json!({ "app": { "connections": [{ "to": target }] } });
        let facts: Facts = serde_json::from_value(json!({
            "schema": 4,
            "hosts": {
                "a": placed("10.0.0.5/24", client, location),
                "b": placed("192.168.1.20/24", web.clone(), Some("home")),
                "c": placed("192.168.1.20/24", web.clone(), Some("lab"))
            }
        }))
        .unwrap();
        build(&facts).map(|m| m.connections[0].to.clone())
    };
    let b = Endpoint::Unit("b".into(), "web".into());
    assert_eq!(
        resolve("http://192.168.1.20:3000", Some("home")).unwrap(),
        b
    );
    assert_eq!(
        resolve("192.168.1.0/24", Some("lab")).unwrap(),
        net("lan @ lab")
    );
    assert_eq!(resolve("lan", Some("home")).unwrap(), net("lan @ home"));
    let err = format!("{:#}", resolve("lan", None).unwrap_err());
    assert!(err.contains("several locations"), "{err}");
}
