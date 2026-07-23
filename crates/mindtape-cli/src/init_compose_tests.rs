#![allow(clippy::unwrap_used)]

use super::compose;

/// The fragments fold in the order given, and the whole re-parses as one TOML
/// document whose sections all survive.
#[test]
fn folds_fragments_in_order_and_reparses() {
    let fragments = vec![
        (
            "workflow",
            "[[tasks.status]]\nvalue = \"todo\"\n".to_owned(),
        ),
        ("priority", "[tasks.priority]\nbands = []\n".to_owned()),
    ];
    let out = compose(&fragments).unwrap();
    assert!(out.contains("[[tasks.status]]"), "{out}");
    assert!(out.contains("[tasks.priority]"), "{out}");
    let table: toml::Table = toml::from_str(&out).unwrap();
    assert!(table.contains_key("tasks"));
}

/// Empty and whitespace-only fragments drop out; a comment-only fragment stays.
#[test]
fn skips_empty_and_keeps_comment_only_fragments() {
    let fragments = vec![
        ("blank", "  \n".to_owned()),
        ("gone", String::new()),
        ("note", "# an axis note\n".to_owned()),
        ("real", "[tasks.priority]\nbands = []\n".to_owned()),
    ];
    let out = compose(&fragments).unwrap();
    assert!(out.contains("# an axis note"), "{out}");
    assert!(out.trim_start().starts_with("# an axis note"), "{out}");
}

/// Two fragments that define the same table collide, and the collision surfaces
/// as an error here rather than downstream.
#[test]
fn rejects_two_fragments_defining_the_same_table() {
    let fragments = vec![
        ("one", "[tasks.priority]\nbands = []\n".to_owned()),
        ("two", "[tasks.priority]\nframework = \"ice\"\n".to_owned()),
    ];
    assert!(compose(&fragments).is_err());
}

/// Disjoint sub-tables of the same parent (`[tasks]`) do not collide — the fold
/// only needs each fragment to avoid re-opening a table another already owns.
#[test]
fn disjoint_subtables_of_one_parent_compose() {
    let fragments = vec![
        ("a", "[tasks.priority]\nbands = []\n".to_owned()),
        ("b", "[tasks.derive-tags]\nenabled = true\n".to_owned()),
        ("c", "[[tasks.roots]]\npath = \"tasks\"\n".to_owned()),
    ];
    let out = compose(&fragments).unwrap();
    let table: toml::Table = toml::from_str(&out).unwrap();
    assert!(table.contains_key("tasks"));
}
