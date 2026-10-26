#![allow(clippy::unwrap_used)]

use serde_json::Value;

use super::{EXPRESSION_TAG, expression_keys, walk_schema};

/// Property names that hold an expression wherever a config struct declares
/// them as a string.
const EXPRESSION_NAMES: [&str; 12] = [
    "when",
    "assert",
    "filter",
    "condition",
    "score",
    "list",
    "required",
    "values-when",
    "default-filter",
    "hide",
    "top",
    "fallback",
];

/// Keys under an expression name that hold no expression: a content rule's
/// `assert` is `"non-empty"` or a table of regexes and word counts.
const NOT_EXPRESSIONS: [&str; 1] = ["tasks.body.content.rules.*.assert"];

/// Each expression key as `dotted.path` and whether it joins kebab names.
fn listed() -> Vec<(String, bool)> {
    expression_keys()
        .iter()
        .map(|key| (key.path.join("."), key.joins_kebab))
        .collect()
}

#[test]
fn the_schema_tags_every_expression_key_with_its_language() {
    let expected = [
        ("commits.rules.*.assert", true),
        ("commits.rules.*.when", true),
        ("ls.columns.*.when", false),
        ("ls.rules.*.condition", false),
        ("ls.rules.*.filter", true),
        ("on.flip.*.actions.*.list", true),
        ("on.flip.*.when", true),
        ("rm.default-filter", true),
        ("tasks.assignee.required", true),
        ("tasks.assignee.values-when", true),
        ("tasks.body.content.rules.*.when", true),
        ("tasks.body.headers.rules.*.when", true),
        ("tasks.difficulty.required", true),
        ("tasks.estimate.required", true),
        ("tasks.fields.*.required", true),
        ("tasks.fields.*.values-when", true),
        ("tasks.priority.frameworks.*.category.*.when", false),
        ("tasks.priority.frameworks.*.score", false),
        ("tasks.priority.required", true),
        ("tasks.rules.*.assert", true),
        ("tasks.rules.*.when", true),
        ("tasks.tags.values-when", true),
        ("views.*.filter", true),
    ];
    let mut listed = listed();
    listed.sort();
    let expected: Vec<(String, bool)> = expected
        .iter()
        .map(|(path, joins)| ((*path).to_owned(), *joins))
        .collect();
    assert_eq!(listed, expected);
}

#[test]
fn every_tag_names_a_known_language() {
    let schema = schemars::schema_for!(crate::config::Config);
    let mut unknown = Vec::new();
    walk_schema(schema.as_value(), &mut |path, node| {
        if let Some(language) = node.get(EXPRESSION_TAG).and_then(Value::as_str)
            && !["filter", "commit", "condition", "formula"].contains(&language)
        {
            unknown.push(format!("{}: {language}", path.join(".")));
        }
    });
    assert!(unknown.is_empty(), "{unknown:?}");
}

/// Whether a schema node takes a string.
fn takes_a_string(node: &serde_json::Map<String, Value>) -> bool {
    match node.get("type") {
        Some(Value::String(kind)) => kind == "string",
        Some(Value::Array(kinds)) => kinds.iter().any(|kind| kind == "string"),
        _ => false,
    }
}

#[test]
fn a_string_under_an_expression_name_carries_the_tag() {
    let schema = schemars::schema_for!(crate::config::Config);
    let tagged: Vec<String> = listed().into_iter().map(|(path, _)| path).collect();
    let mut untagged = Vec::new();
    walk_schema(schema.as_value(), &mut |path, node| {
        let named = path
            .last()
            .is_some_and(|name| EXPRESSION_NAMES.contains(&name.as_str()));
        let dotted = path.join(".");
        let exempt = NOT_EXPRESSIONS.contains(&dotted.as_str());
        if named && takes_a_string(node) && !tagged.contains(&dotted) && !exempt {
            untagged.push(dotted);
        }
    });
    assert!(
        untagged.is_empty(),
        "tag these with #[schemars(extend(\"x-expression\" = ...))]: {untagged:?}"
    );
}
