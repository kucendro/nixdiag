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
