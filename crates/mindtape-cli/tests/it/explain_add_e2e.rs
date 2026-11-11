//! `mt explain add`: the merged `[[on.add]]` hooks, their actions and the
//! layer or pack that supplied each one.
#![allow(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing
)]

use mindtape::test_support::write_file;

use crate::support;

/// A pack with a hook the project replaces, one it disables and one it keeps.
const PACK: &str = r#"
[[on.add]]
name = "default"
actions = [{ dir = "tasks/pack" }]

[[on.add]]
name = "unused"
actions = [{ dir = "tasks/unused" }]

[[on.add]]
name = "kept"
actions = [{ dir = "tasks/kept" }]
"#;

const CONFIG: &str = r#"extends = ["./pack.toml"]
disable = ["unused"]

[tasks]
roots = ["tasks"]

[[on.add]]
name = "default"
actions = [{ dir = "tasks/project" }]

[[on.add]]
name = "fixes"
when = 'template == "task"'
actions = [{ set = { kind = "bug" } }]
"#;

fn project() -> tempfile::TempDir {
    let dir = support::scaffold_project(CONFIG);
    write_file(dir.path(), "pack.toml", PACK);
    dir
}

fn stdout(dir: &std::path::Path, args: &[&str]) -> String {
    let out = support::mt(dir).args(args).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn explain_add_lists_the_merged_hooks_with_their_origin() {
    let dir = project();
    let text = stdout(dir.path(), &["explain", "add"]);
    assert!(text.contains("default"), "{text}");
    assert!(text.contains("tasks/project"), "{text}");
    assert!(!text.contains("tasks/pack"), "{text}");
    assert!(!text.contains("unused"), "{text}");
    assert!(text.contains("kept"), "{text}");
    assert!(text.contains("tasks/kept"), "{text}");
    assert!(text.contains("fixes"), "{text}");
    assert!(text.contains("kind = \"bug\""), "{text}");
    assert!(
        text.contains("pack.toml"),
        "a kept pack hook names its pack:\n{text}"
    );
}

#[test]
fn explain_add_json_uses_its_schema_and_names_actions() {
    let dir = project();
    let json: serde_json::Value =
        serde_json::from_str(&stdout(dir.path(), &["explain", "add", "--json"])).unwrap();
    mindtape::test_support::assert_schema(&json, "explain-add");
    let hooks = json["hooks"].as_array().unwrap();
    assert_eq!(hooks.len(), 3, "{json}");
    assert_eq!(hooks[0]["label"], "on.add.default");
    assert_eq!(hooks[0]["name"], "default");
    assert_eq!(hooks[0]["actions"][0], "dir = \"tasks/project\"");
    assert_eq!(hooks[2]["label"], "on.add.fixes");
    assert_eq!(hooks[2]["name"], "fixes");
}

#[test]
fn config_show_origin_labels_each_add_hook() {
    let dir = project();
    let text = stdout(dir.path(), &["config", "show", "--origin"]);
    for label in [
        "rule on.add.default:",
        "rule on.add.kept:",
        "rule on.add.fixes:",
    ] {
        assert!(text.contains(label), "missing {label:?} in:\n{text}");
    }
    assert!(!text.contains("on.add.unused"), "{text}");
}
