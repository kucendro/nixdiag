use crate::api::{self, API_VERSION};
use crate::render::out::JSON_MARKER;
use schemars::{schema_for, JsonSchema};
use serde_json::{json, Map, Value};

fn repoint_refs(v: &mut Value) {
    match v {
        Value::Object(map) => {
            if let Some(Value::String(r)) = map.get_mut("$ref") {
                if let Some(name) = r.strip_prefix("#/$defs/") {
                    *r = format!("#/components/schemas/{name}");
                }
            }
            for (_, child) in map.iter_mut() {
                repoint_refs(child);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(repoint_refs),
        _ => {}
    }
}

fn add<T: JsonSchema>(name: &str, schemas: &mut Map<String, Value>) {
    let mut root = serde_json::to_value(schema_for!(T)).unwrap_or(Value::Null);
    let Some(obj) = root.as_object_mut() else {
        return;
    };
    obj.remove("$schema");
    obj.remove("title");
    if let Some(Value::Object(defs)) = obj.remove("$defs") {
        for (k, mut v) in defs {
            repoint_refs(&mut v);
            schemas.insert(k, v);
        }
    }
    repoint_refs(&mut root);
    schemas.insert(name.to_string(), root);
}

fn get(summary: &str, schema: &str) -> Value {
    json!({
        "get": {
            "summary": summary,
            "responses": {
                "200": {
                    "description": summary,
                    "content": {
                        "application/json": {
                            "schema": { "$ref": format!("#/components/schemas/{schema}") }
                        }
                    }
                }
            }
        }
    })
}

pub(super) fn build(has_lock: bool, has_closures: bool) -> Value {
    let mut schemas = Map::new();
    add::<api::Index>("Index", &mut schemas);
    add::<api::Hosts>("Hosts", &mut schemas);
    add::<api::Services>("Services", &mut schemas);
    add::<api::Topology>("Topology", &mut schemas);
    add::<api::Snapshot>("Snapshot", &mut schemas);
    if has_lock {
        add::<api::Inputs>("Inputs", &mut schemas);
    }
    if has_closures {
        add::<api::Closures>("Closures", &mut schemas);
    }

    let base = format!("/api/{API_VERSION}");
    let mut paths = Map::new();
    paths.insert(
        format!("{base}/index.json"),
        get("The endpoints this build published", "Index"),
    );
    paths.insert(
        format!("{base}/hosts.json"),
        get("Hosts, platforms, open ports and users", "Hosts"),
    );
    paths.insert(
        format!("{base}/services.json"),
        get("Services this repo configures and where", "Services"),
    );
    paths.insert(
        format!("{base}/topology.json"),
        get("Annotated nodes, edges and endpoints", "Topology"),
    );
    if has_lock {
        paths.insert(
            format!("{base}/inputs.json"),
            get("Flake inputs, lock dates and duplicates", "Inputs"),
        );
    }
    if has_closures {
        paths.insert(
            format!("{base}/closures.json"),
            get("Per-host closure sizes by package", "Closures"),
        );
    }
    paths.insert(
        format!("{base}/snapshot.json"),
        get("Totals plus revision identity, for trends", "Snapshot"),
    );

    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "nixdiag",
            "version": env!("CARGO_PKG_VERSION"),
            "description": format!(
                "{JSON_MARKER}\n\n\
                 Read-only static documents describing what a Nix flake declares. \
                 Every path is a file: GET only, no parameters, no auth. \
                 Readers should tolerate unknown keys and treat an unrecognised \
                 `meta.schema` as newer than they understand."
            ),
        },
        "paths": paths,
        "components": { "schemas": schemas },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refs_are_repointed_at_the_component_space() {
        let mut v = json!({
            "properties": { "meta": { "$ref": "#/$defs/Meta" } },
            "items": [ { "$ref": "#/$defs/HostEntry" } ],
            "unrelated": "#/$defs/NotARef",
        });
        repoint_refs(&mut v);
        assert_eq!(v["properties"]["meta"]["$ref"], "#/components/schemas/Meta");
        assert_eq!(v["items"][0]["$ref"], "#/components/schemas/HostEntry");
        assert_eq!(v["unrelated"], "#/$defs/NotARef");
    }

    #[test]
    fn the_document_has_no_dangling_refs_and_carries_the_marker() {
        let doc = build(true, true);
        let text = serde_json::to_string(&doc).unwrap();
        assert!(text.contains(crate::render::out::MARKER_WORD));
        assert!(!text.contains("#/$defs/"), "unhoisted subschema ref");

        let schemas = doc["components"]["schemas"].as_object().unwrap();
        for name in ["Index", "Hosts", "Services", "Topology", "Snapshot"] {
            assert!(schemas.contains_key(name), "missing schema {name}");
        }
        for cap in text.split("#/components/schemas/").skip(1) {
            let name: String = cap.chars().take_while(|c| c.is_alphanumeric()).collect();
            assert!(schemas.contains_key(&name), "dangling ref to {name}");
        }
    }

    #[test]
    fn absent_inputs_and_closures_drop_their_paths() {
        let doc = build(false, false);
        let paths = doc["paths"].as_object().unwrap();
        assert!(!paths.contains_key("/api/v1/inputs.json"));
        assert!(!paths.contains_key("/api/v1/closures.json"));
        let schemas = doc["components"]["schemas"].as_object().unwrap();
        assert!(!schemas.contains_key("Inputs"));
        assert!(!schemas.contains_key("Closures"));
    }
}
