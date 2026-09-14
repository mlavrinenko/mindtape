//! `[tasks.body.content.rules]` — section-content gate parsing: the assertion
//! shapes (string `"non-empty"` vs. the one-key tables `min-words`,
//! `must-match`, `must-not-match`), path reuse, and the malformed-shape error
//! message.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::ContentAssert;
use crate::config::{Config, HeaderPathSegment, parse_config};

#[test]
fn parses_non_empty_assertion() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = \"non-empty\"\n";
    let config = parse_config(text).unwrap();
    let rules = &config.tasks.body.content.rules;
    assert_eq!(rules.len(), 1);
    assert_eq!(
        rules[0].path,
        vec![HeaderPathSegment::Exact("Summary".to_owned())]
    );
    assert_eq!(rules[0].assert, ContentAssert::NonEmpty);
    assert!(rules[0].when.is_none());
    assert!(rules[0].message.is_none());
}

#[test]
fn parses_min_words_assertion() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Scope\"]\nassert = { min-words = 20 }\n";
    let config = parse_config(text).unwrap();
    assert_eq!(
        config.tasks.body.content.rules[0].assert,
        ContentAssert::MinWords(20)
    );
}

#[test]
fn parses_max_words_assertion() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Scope\"]\nassert = { max-words = 200 }\n";
    let config = parse_config(text).unwrap();
    assert_eq!(
        config.tasks.body.content.rules[0].assert,
        ContentAssert::MaxWords(200)
    );
}

#[test]
fn parses_regex_assertions() {
    let text = r#"
[[tasks.body.content.rules]]
path   = ["Watch for"]
assert = { must-match = "\\?" }
message = "an open-question section states its question"

[[tasks.body.content.rules]]
path   = ["Scope"]
assert = { must-not-match = "TODO" }
when   = 'has_tag("bug")'
"#;
    let config = parse_config(text).unwrap();
    let rules = &config.tasks.body.content.rules;
    assert_eq!(rules[0].assert, ContentAssert::MustMatch("\\?".to_owned()));
    assert_eq!(
        rules[0].message.as_deref(),
        Some("an open-question section states its question")
    );
    assert_eq!(
        rules[1].assert,
        ContentAssert::MustNotMatch("TODO".to_owned())
    );
    assert_eq!(rules[1].when.as_deref(), Some("has_tag(\"bug\")"));
}

#[test]
fn parses_regex_path_segment_in_content_rule() {
    let text =
        "[[tasks.body.content.rules]]\npath = [{ match = \"^Watch\" }]\nassert = \"non-empty\"\n";
    let config = parse_config(text).unwrap();
    assert_eq!(
        config.tasks.body.content.rules[0].path[0],
        HeaderPathSegment::Pattern {
            pattern: "^Watch".to_owned(),
            recursive: false,
        }
    );
}

#[test]
fn rejects_unknown_assertion_string() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = \"long\"\n";
    let err = parse_config(text).unwrap_err().to_string();
    assert!(
        err.contains("invalid assertion `long`"),
        "expected assertion-shape error, got: {err}"
    );
}

#[test]
fn rejects_unknown_assertion_key() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = { long = 10 }\n";
    assert!(parse_config(text).is_err());
}

#[test]
fn rejects_assertion_table_with_two_keys() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = { min-words = 1, must-match = \"x\" }\n";
    let err = parse_config(text).unwrap_err().to_string();
    assert!(
        err.contains("exactly one"),
        "expected single-key error, got: {err}"
    );
}

#[test]
fn assertion_type_error_names_expected_shape() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = 42\n";
    let err = parse_config(text).unwrap_err().to_string();
    assert!(
        err.contains("non-empty") && err.contains("min-words"),
        "expected assertion-shape hint, got: {err}"
    );
}

#[test]
fn content_rules_default_to_empty() {
    let config = parse_config("").unwrap();
    assert!(config.tasks.body.content.rules.is_empty());
}

#[test]
fn rejects_unknown_content_rule_field() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Summary\"]\nassert = \"non-empty\"\npattern = \"x\"\n";
    assert!(parse_config(text).is_err());
}

#[test]
fn headers_and_content_rules_coexist() {
    let text = r#"
[[tasks.body.headers.rules]]
path = ["Summary"]
action = "unique"

[[tasks.body.content.rules]]
path = ["Summary"]
assert = "non-empty"
"#;
    let config: Config = parse_config(text).unwrap();
    assert_eq!(config.tasks.body.headers.rules.len(), 1);
    assert_eq!(config.tasks.body.content.rules.len(), 1);
}
