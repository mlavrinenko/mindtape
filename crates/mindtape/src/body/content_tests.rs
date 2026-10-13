//! `[tasks.body.content.rules]` — section-content gate parsing: the assertion
//! shapes (string `"non-empty"` vs. the one-key tables `min-words`,
//! `must-match`, `must-not-match`, `density`), path reuse, and the
//! malformed-shape error message.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{ContentAssert, ContentRuleConfig, DensitySpec, DensityUnit};
use crate::config::{Config, HeaderPathSegment, parse_config};

/// The first `[[tasks.body.content.rules]]` entry `text` declares.
fn first_rule(text: &str) -> ContentRuleConfig {
    parse_config(text)
        .unwrap()
        .tasks
        .body
        .content
        .rules
        .remove(0)
}

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
    let rule = first_rule(text);
    assert_eq!(rule.assert, ContentAssert::MinWords(20));
}

#[test]
fn parses_max_words_assertion() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Scope\"]\nassert = { max-words = 200 }\n";
    let rule = first_rule(text);
    assert_eq!(rule.assert, ContentAssert::MaxWords(200));
}

#[test]
fn parses_density_assertion() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Scope\"]\nassert = { density = { pattern = \"—\", max = 1, per = \"paragraph\" } }\n";
    let rule = first_rule(text);
    assert_eq!(
        rule.assert,
        ContentAssert::Density(DensitySpec {
            pattern: "—".to_owned(),
            max: 1,
            per: DensityUnit::Paragraph,
        })
    );
}

#[test]
fn density_per_defaults_to_section() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Scope\"]\nassert = { density = { pattern = \"—\", max = 1 } }\n";
    let rule = first_rule(text);
    assert_eq!(
        rule.assert,
        ContentAssert::Density(DensitySpec {
            pattern: "—".to_owned(),
            max: 1,
            per: DensityUnit::Section,
        })
    );
}

#[test]
fn rejects_bad_density_per_value() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Scope\"]\nassert = { density = { pattern = \"—\", max = 1, per = \"word\" } }\n";
    let err = parse_config(text).unwrap_err().to_string();
    assert!(
        err.contains("invalid density unit `word`"),
        "expected a density-unit error, got: {err}"
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
    let rule = first_rule(text);
    assert_eq!(
        rule.path[0],
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
fn parses_content_rule_name() {
    let text = "[[tasks.body.content.rules]]\nname = \"problem-steps\"\npath = [\"Problem\"]\nassert = \"non-empty\"\n";
    let rule = first_rule(text);
    assert_eq!(rule.name.as_deref(), Some("problem-steps"));
}

#[test]
fn content_rule_name_defaults_to_none() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Problem\"]\nassert = \"non-empty\"\n";
    let rule = first_rule(text);
    assert!(rule.name.is_none());
}

#[test]
fn content_rule_label_prefers_name_over_position() {
    let text = "[[tasks.body.content.rules]]\nname = \"problem-steps\"\npath = [\"Problem\"]\nassert = \"non-empty\"\n";
    let rule = first_rule(text);
    assert_eq!(rule.label(3), "body.content.rules.problem-steps");
}

#[test]
fn content_rule_label_falls_back_to_position() {
    let text = "[[tasks.body.content.rules]]\npath = [\"Problem\"]\nassert = \"non-empty\"\n";
    let config = parse_config(text).unwrap();
    assert_eq!(
        config.tasks.body.content.rules[0].label(3),
        "body.content.rules[3]"
    );
}

#[test]
fn rejects_duplicate_content_rule_names() {
    let text = "[[tasks.body.content.rules]]\nname = \"dup\"\npath = [\"A\"]\nassert = \"non-empty\"\n\n\
                [[tasks.body.content.rules]]\nname = \"dup\"\npath = [\"B\"]\nassert = \"non-empty\"\n";
    let err = parse_config(text).unwrap().validate().unwrap_err();
    assert!(
        err.contains("tasks.body.content.rules[1]") && err.contains("dup"),
        "{err}"
    );
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
