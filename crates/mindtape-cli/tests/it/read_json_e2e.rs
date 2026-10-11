//! `mt graph --json` and `mt preset ls --json` print JSON on stdout.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::disallowed_methods
)]

use crate::support;

use support::{mt, project_with_tasks};

fn json(dir: &std::path::Path, args: &[&str]) -> serde_json::Value {
    let out = mt(dir).args(args).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn graph_json_lists_nodes_and_typed_edges() {
    let dir = project_with_tasks(&[("alpha.typ", "Alpha"), ("beta.typ", "Beta")]);
    let beta = dir.path().join("tasks/beta.typ");
    let text = std::fs::read_to_string(&beta).unwrap();
    std::fs::write(
        &beta,
        text.replace("status:", "links: depends-on(\"alpha.typ\"),\n  status:"),
    )
    .unwrap();

    let value = json(dir.path(), &["graph", "--json"]);

    assert_eq!(value["schema_version"], mindtape::SCHEMA_VERSION);
    let ids: Vec<_> = value["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 2, "{value}");
    assert!(ids.contains(&"alpha") && ids.contains(&"beta"), "{value}");
    assert!(value["nodes"][0]["title"].is_string());
    assert_eq!(
        value["edges"],
        serde_json::json!([{"from": "beta", "to": "alpha", "kind": "depends-on", "note": null}])
    );
    assert_eq!(value["external"], serde_json::json!([]));
}

#[test]
fn preset_ls_json_names_each_preset_with_origin_and_description() {
    let dir = project_with_tasks(&[]);

    let value = json(dir.path(), &["preset", "ls", "--json"]);

    let presets = value["presets"].as_array().unwrap();
    let default = presets.iter().find(|p| p["default"] == true).unwrap();
    assert_eq!(default["origin"], "built-in");
    assert!(default["description"].is_string(), "{default}");
    assert!(presets.iter().all(|p| p["name"].is_string()), "{value}");
}
