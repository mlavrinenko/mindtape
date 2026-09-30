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

/// Look up a dotted path in a parsed TOML value, returning the leaf value.
fn dig<'a>(value: &'a toml::Value, path: &str) -> Option<&'a toml::Value> {
    let mut cursor = value;
    for key in path.split('.') {
        cursor = cursor.get(key)?;
    }
    Some(cursor)
}

/// An `overlay` fragment merges last and beats every ordinary fragment: it
/// overrides a scalar the base set and adds a section the axes never touched,
/// and the flattened output carries no literal `[overlay]`.
#[test]
fn overlay_lands_last_and_beats_the_base() {
    let fragments = vec![
        ("recurrence", "[tasks.recur]\nrrule = false\n".to_owned()),
        (
            "overlay",
            "[tasks.recur]\nrrule = true\n[[tasks.links.kinds]]\nkind = \"spike\"\n".to_owned(),
        ),
    ];
    let out = compose(&fragments).unwrap();
    assert!(!out.contains("[overlay]"), "{out}");
    let table: toml::Value = toml::from_str(&out).unwrap();
    let recur = dig(&table, "tasks.recur.rrule").unwrap().as_bool().unwrap();
    assert!(recur, "overlay must win the scalar: {out}");
    // The overlay's own section survives the flatten.
    assert!(dig(&table, "tasks.links.kinds").is_some(), "{out}");
}

/// The overlay merges the base table key-by-key (a scalar it does not mention is
/// kept) while replacing an array wholesale — the reference deep-merge.
#[test]
fn overlay_merges_tables_and_replaces_arrays() {
    let fragments = vec![
        (
            "priority",
            "[tasks.priority]\nframework = \"ice\"\nbands = [\"low\", \"high\"]\n".to_owned(),
        ),
        (
            "overlay",
            "[tasks.priority]\nbands = [\"only\"]\n".to_owned(),
        ),
    ];
    let out = compose(&fragments).unwrap();
    let table: toml::Value = toml::from_str(&out).unwrap();
    // The untouched scalar key stays.
    assert_eq!(
        dig(&table, "tasks.priority.framework").unwrap().as_str(),
        Some("ice"),
        "{out}"
    );
    // The array is replaced wholesale, not appended.
    let bands = dig(&table, "tasks.priority.bands")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(bands.len(), 1, "{out}");
    assert_eq!(bands.first().unwrap().as_str(), Some("only"), "{out}");
}

/// The overlay is exempt from the array-conflict rule: it may redeclare an array
/// an ordinary fragment already owns, precisely to override it.
#[test]
fn overlay_may_redeclare_an_array_the_base_owns() {
    let fragments = vec![
        (
            "workflow",
            "[[tasks.status]]\nvalue = \"todo\"\n".to_owned(),
        ),
        ("overlay", "[[tasks.status]]\nvalue = \"open\"\n".to_owned()),
    ];
    let out = compose(&fragments).unwrap();
    let table: toml::Value = toml::from_str(&out).unwrap();
    let status = dig(&table, "tasks.status").unwrap().as_array().unwrap();
    assert_eq!(status.len(), 1, "overlay replaces the legend: {out}");
    assert_eq!(
        status.first().unwrap().get("value").unwrap().as_str(),
        Some("open"),
        "{out}"
    );
}

/// A fragment that is only an overlay composes to just the overlay — the
/// preset-with-only-an-overlay case, where the base is empty.
#[test]
fn overlay_only_composes_to_the_overlay() {
    let fragments = vec![(
        "overlay",
        "[tasks]\nroots = [\"tasks\"]\n[views.mine]\nfilter = \"!terminal\"\n".to_owned(),
    )];
    let out = compose(&fragments).unwrap();
    let table: toml::Value = toml::from_str(&out).unwrap();
    assert_eq!(
        dig(&table, "tasks.roots")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1,
        "{out}"
    );
    assert!(dig(&table, "views.mine").is_some(), "{out}");
}

/// Two ordinary fragments that each declare the `status` legend hard-error, and
/// the message names both fragments and the section — never last-wins.
#[test]
fn two_status_fragments_error_naming_both() {
    let fragments = vec![
        (
            "workflow",
            "[[tasks.status]]\nvalue = \"todo\"\n".to_owned(),
        ),
        (
            "imported",
            "[[tasks.status]]\nvalue = \"open\"\n".to_owned(),
        ),
    ];
    let err = compose(&fragments).unwrap_err().to_string();
    assert!(err.contains("workflow"), "{err}");
    assert!(err.contains("imported"), "{err}");
    assert!(err.contains("tasks.status"), "{err}");
}

/// The array-conflict rule reaches a nested array too: priority bands declared
/// by two fragments collide and name the nested path.
#[test]
fn two_priority_bands_fragments_error() {
    let fragments = vec![
        ("a", "[tasks.priority]\nbands = [\"low\"]\n".to_owned()),
        ("b", "[tasks.priority]\nbands = [\"high\"]\n".to_owned()),
    ];
    let err = compose(&fragments).unwrap_err().to_string();
    assert!(err.contains("tasks.priority.bands"), "{err}");
}

/// The fold joins fragments with exactly one blank line between them and never
/// opens the document with one. Inverting the emptiness guard would put the
/// leading newline on the first fragment instead of the separator.
#[test]
fn fragments_join_with_one_blank_line_and_no_leading_newline() {
    let out = compose(&[("a", "x = 1\n".to_owned()), ("b", "y = 2\n".to_owned())]).unwrap();
    assert_eq!(out, "x = 1\n\ny = 2\n");
}

/// The array-conflict guard names *two different* fragments: the same fragment
/// name appearing twice is not a collision, and its arrays append. Forcing the
/// guard to `true` would reject a repeated fragment name.
#[test]
fn the_same_fragment_name_may_repeat_an_array_section() {
    let out = compose(&[
        ("same", "[[tasks.status]]\nvalue = \"a\"\n".to_owned()),
        ("same", "[[tasks.status]]\nvalue = \"b\"\n".to_owned()),
    ])
    .unwrap();
    let table: toml::Value = toml::from_str(&out).unwrap();
    let status = table
        .get("tasks")
        .and_then(|tasks| tasks.get("status"))
        .and_then(toml::Value::as_array)
        .unwrap_or_else(|| panic!("status legend missing: {out}"));
    assert_eq!(status.len(), 2, "{out}");
}

/// Two fragments declaring different array sections compose cleanly — the rule
/// fires on a shared array, not on arrays in general.
#[test]
fn different_array_sections_compose() {
    let fragments = vec![
        (
            "workflow",
            "[[tasks.status]]\nvalue = \"todo\"\n".to_owned(),
        ),
        (
            "priority",
            "[tasks.priority]\nbands = [\"low\"]\n".to_owned(),
        ),
        (
            "organisation",
            "[[tasks.roots]]\npath = \"tasks\"\n".to_owned(),
        ),
    ];
    let out = compose(&fragments).unwrap();
    let table: toml::Table = toml::from_str(&out).unwrap();
    assert!(table.contains_key("tasks"));
}
