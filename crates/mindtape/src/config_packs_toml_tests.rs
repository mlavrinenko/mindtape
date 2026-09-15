#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{read_array, read_string_list, remove_path, write_array};

fn table(text: &str) -> toml::Table {
    toml::from_str(text).unwrap()
}

#[test]
fn read_string_list_defaults_to_empty_when_absent() {
    let t = table("");
    assert_eq!(
        read_string_list(&t, "extends").unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn read_string_list_reads_declared_entries() {
    let t = table("extends = [\"a\", \"b\"]\n");
    assert_eq!(read_string_list(&t, "extends").unwrap(), ["a", "b"]);
}

#[test]
fn read_string_list_rejects_a_non_array() {
    let t = table("extends = \"a\"\n");
    let err = read_string_list(&t, "extends").unwrap_err();
    assert!(err.to_string().contains("extends"), "{err}");
}

#[test]
fn read_string_list_rejects_a_non_string_entry() {
    let t = table("extends = [1]\n");
    let err = read_string_list(&t, "extends").unwrap_err();
    assert!(err.to_string().contains("entries must be strings"), "{err}");
}

#[test]
fn read_array_returns_empty_for_a_missing_path() {
    let t = table("");
    assert!(read_array(&t, &["tasks", "rules"]).is_empty());
}

#[test]
fn read_array_returns_empty_when_an_ancestor_is_not_a_table() {
    let t = table("tasks = 1\n");
    assert!(read_array(&t, &["tasks", "rules"]).is_empty());
}

#[test]
fn read_array_reads_a_nested_array() {
    let t = table("[[tasks.rules]]\nname = \"a\"\nassert = \"true\"\n");
    let values = read_array(&t, &["tasks", "rules"]);
    assert_eq!(values.len(), 1);
    assert_eq!(
        values[0].get("name").and_then(toml::Value::as_str),
        Some("a")
    );
}

#[test]
fn remove_path_drops_the_leaf_key_only() {
    let mut t =
        table("[[tasks.rules]]\nname = \"a\"\nassert = \"true\"\n[tasks]\nroots = [\"x\"]\n");
    remove_path(&mut t, &["tasks", "rules"]);
    assert!(read_array(&t, &["tasks", "rules"]).is_empty());
    assert_eq!(
        t["tasks"]["roots"].as_array().unwrap()[0].as_str(),
        Some("x")
    );
}

#[test]
fn remove_path_on_a_missing_path_is_a_no_op() {
    let mut t = table("");
    remove_path(&mut t, &["tasks", "rules"]);
    assert!(t.is_empty());
}

#[test]
fn write_array_creates_missing_intermediate_tables() {
    let mut t = table("");
    write_array(
        &mut t,
        &["tasks", "body", "headers", "rules"],
        vec![toml::Value::Boolean(true)],
    )
    .unwrap();
    assert_eq!(
        read_array(&t, &["tasks", "body", "headers", "rules"]),
        vec![toml::Value::Boolean(true)]
    );
}

#[test]
fn write_array_overwrites_an_existing_array() {
    let mut t = table("[[tasks.rules]]\nname = \"old\"\nassert = \"true\"\n");
    write_array(&mut t, &["tasks", "rules"], vec![]).unwrap();
    assert!(read_array(&t, &["tasks", "rules"]).is_empty());
}

#[test]
fn write_array_rejects_a_non_table_ancestor() {
    let mut t = table("tasks = 1\n");
    let err = write_array(&mut t, &["tasks", "rules"], vec![]).unwrap_err();
    assert!(err.to_string().contains("tasks"), "{err}");
}
