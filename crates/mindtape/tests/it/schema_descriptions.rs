//! Every description in the config and `at.toml` schemas is hover text in an
//! editor, and the config schema's are cells on the docs site's Configuration
//! page, so each must read as user prose.
//! A Rust path or a rustdoc intra-doc link in one means a developer note
//! leaked out of a doc comment; that note belongs in a `//` comment instead.
#![allow(clippy::unwrap_used)]

use serde_json::Value;

/// Collect `(key path, description)` for every `description` string in the
/// schema, depth-first, with the key path joined by `.`.
fn descriptions(node: &Value, path: &str, out: &mut Vec<(String, String)>) {
    let join = |key: &str| {
        if path.is_empty() {
            key.to_owned()
        } else {
            format!("{path}.{key}")
        }
    };
    match node {
        Value::Object(map) => {
            if let Some(Value::String(text)) = map.get("description") {
                out.push((path.to_owned(), text.clone()));
            }
            for (key, child) in map {
                descriptions(child, &join(key), out);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                descriptions(child, &join(&index.to_string()), out);
            }
        }
        _ => {}
    }
}

/// Whether `text` carries a rustdoc intra-doc link: `` [`name`] `` with no
/// `(url)` after it.
fn has_intra_doc_link(text: &str) -> bool {
    let mut rest = text;
    while let Some(start) = rest.find("[`") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("`]") else {
            return false;
        };
        let name = &after[..end];
        let tail = &after[end + 2..];
        if !name.contains('`') && !tail.starts_with('(') {
            return true;
        }
        rest = after;
    }
    false
}

/// The reason a description fails, or `None` when it reads as user prose.
fn problem(text: &str) -> Option<&'static str> {
    if text.contains("crate::") {
        Some("names a `crate::` path")
    } else if has_intra_doc_link(text) {
        Some("carries a rustdoc intra-doc link")
    } else {
        None
    }
}

#[test]
fn schema_descriptions_carry_no_rust_paths_or_intra_doc_links() {
    let mut found = Vec::new();
    for text in [mindtape::config_schema_json(), mindtape::at::schema_json()] {
        let schema: Value = serde_json::from_str(&text).unwrap();
        let before = found.len();
        descriptions(&schema, "", &mut found);
        assert!(found.len() > before, "a schema carries no descriptions");
    }
    let failures: Vec<String> = found
        .iter()
        .filter_map(|(path, text)| problem(text).map(|why| format!("{path}: {why}")))
        .collect();
    assert!(
        failures.is_empty(),
        "move these developer notes to `//` comments:\n{}",
        failures.join("\n")
    );
}

#[test]
fn intra_doc_link_detection() {
    assert!(has_intra_doc_link("see [`validate_rules`] for more"));
    assert!(has_intra_doc_link("[`crate::config_packs`]"));
    assert!(!has_intra_doc_link("the `[ls] hyperlinks` key"));
    assert!(!has_intra_doc_link("`[[tasks.rules]]` entries"));
    assert!(!has_intra_doc_link("[`mt ls`](https://example.org/cli/)"));
    assert!(!has_intra_doc_link("plain prose"));
}
