use super::*;
use crate::render::d2::{Diagram, Doc};
use serde_json::json;

fn drawn(board: impl Fn(&View) -> Doc) -> String {
    let unit = |to: &str, plane: &str| json!({ "connections": [{ "to": to, "label": plane, "plane": plane }] });
    let facts: Facts = serde_json::from_value(json!({
        "schema": 4,
        "hosts": {
            "a": { "kind": "nixos", "topology": { "units": {
                "app": unit("b/db", "data"),
                "vpn": unit("b/ctl", "control"),
                "agent": unit("b/mon", "mgmt")
            } } },
            "b": { "kind": "nixos", "topology": { "units": { "db": {}, "ctl": {}, "mon": {} } } }
        }
    }))
    .unwrap();
    let model = crate::topology::build(&facts).unwrap();
    board(&View::new(&facts, &model)).to_string()
}

fn draw(d: &impl Diagram) -> Doc {
    let mut doc = Doc::default();
    d.draw(&mut doc);
    doc
}

#[test]
fn overview_keeps_data_and_boards_keep_every_plane() {
    let overview = drawn(|v| draw(&Overview(v)));
    assert!(overview.contains("{class: flow}"), "{overview}");
    assert!(
        !overview.contains("control") && !overview.contains("mgmt"),
        "{overview}"
    );

    let board = drawn(|v| draw(&HostBoard { view: v, host: "a" }));
    assert!(board.contains(": \"control\" {class: control}"), "{board}");
    assert!(board.contains(": \"mgmt\" {class: mgmt}"), "{board}");
    assert!(board.contains(": \"data\" {class: flow}"), "{board}");
}

#[test]
fn overlay_draws_members_routes_exits_and_the_control_server() {
    let facts: Facts = serde_json::from_value(json!({
        "schema": 4,
        "hosts": {
            "a": {
                "kind": "nixos",
                "network": { "interfaces": { "ts0": { "kind": "mesh", "server": "hs", "routes": ["10.1.0.0/16", "0.0.0.0/0"] } } },
                "topology": {
                    "networks": { "hs": { "cidrs": ["100.64.0.0/10"], "kind": "mesh", "server": "hs" } },
                    "units": { "vpn": { "connections": [{ "to": "b/ctl", "label": "login", "plane": "control" }] } }
                }
            },
            "b": { "kind": "nixos", "topology": { "units": { "ctl": {} } } }
        }
    }))
    .unwrap();
    let model = crate::topology::build(&facts).unwrap();
    let overlay = draw(&Overlay(&View::new(&facts, &model))).to_string();
    for line in [
        "\"net hs\" -- \"a\": \"ts0\" {class: mesh}",
        "\"a\" -- \"net 10.1.0.0/16\": \"routes via ts0\" {class: route}",
        "\"a\" -- \":internet\": \"routes via ts0\" {class: route}",
        "\"a\" -> \"b\": \"login\" {class: control}",
        "\"b\": \"🖥️ b\\ncontrol server\"",
    ] {
        assert!(overlay.contains(line), "{line}\n{overlay}");
    }
}

#[test]
fn a_dual_stack_route_is_drawn_once() {
    let facts: Facts = serde_json::from_value(json!({
        "schema": 4,
        "hosts": { "a": { "kind": "nixos", "network": { "interfaces": { "eth0": {
            "addresses": [
                { "cidr": "192.168.1.10/24", "scope": "lan" },
                { "cidr": "fd00::10/64", "scope": "lan" }
            ],
            "routes": ["10.0.0.0/8"]
        } } } } }
    }))
    .unwrap();
    let model = crate::topology::build(&facts).unwrap();
    let board = draw(&Networks(&View::new(&facts, &model))).to_string();
    assert_eq!(board.matches("routes via eth0").count(), 1, "{board}");
}
