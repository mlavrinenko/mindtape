//! `[tasks.body.headers.rules]` — section-heading gate parsing: path segment
//! shapes (exact string vs. `{ match, recursive }` pattern), the
//! require/forbid action enum, and the malformed-shape error message.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{HeaderAction, HeaderPathSegment, parse_config};

#[test]
fn parses_header_require_rule() {
    let config =
        parse_config("[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n")
            .unwrap();
    let rules = &config.tasks.body.headers.rules;
    assert_eq!(rules.len(), 1);
    assert_eq!(
        rules[0].path,
        vec![HeaderPathSegment::Exact("Summary".to_owned())]
    );
    assert_eq!(rules[0].action, HeaderAction::Require);
    assert!(rules[0].when.is_none());
    assert!(rules[0].message.is_none());
}

#[test]
fn parses_header_mixed_path_and_when() {
    let text = r#"
[[tasks.body.headers.rules]]
path    = ["Approach", { match = "^Phase", recursive = true }]
action  = "forbid"
when    = 'has_tag("milestone")'
message = "no phases"
"#;
    let config = parse_config(text).unwrap();
    let rule = &config.tasks.body.headers.rules[0];
    assert_eq!(rule.path.len(), 2);
    assert_eq!(
        rule.path[0],
        HeaderPathSegment::Exact("Approach".to_owned())
    );
    assert_eq!(
        rule.path[1],
        HeaderPathSegment::Pattern {
            pattern: "^Phase".to_owned(),
            recursive: true,
        }
    );
    assert_eq!(rule.action, HeaderAction::Forbid);
    assert_eq!(rule.when.as_deref(), Some("has_tag(\"milestone\")"));
    assert_eq!(rule.message.as_deref(), Some("no phases"));
}

#[test]
fn parses_pattern_segment_without_recursive() {
    let text =
        "[[tasks.body.headers.rules]]\npath = [{ match = \"^Phase\" }]\naction = \"forbid\"\n";
    let config = parse_config(text).unwrap();
    assert_eq!(
        config.tasks.body.headers.rules[0].path[0],
        HeaderPathSegment::Pattern {
            pattern: "^Phase".to_owned(),
            recursive: false,
        }
    );
}

#[test]
fn rejects_invalid_header_action() {
    let text = "[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"unknown\"\n";
    assert!(parse_config(text).is_err());
}

#[test]
fn rejects_pattern_segment_without_match_key() {
    let text = "[[tasks.body.headers.rules]]\npath = [{ recursive = true }]\naction = \"forbid\"\n";
    assert!(parse_config(text).is_err());
}

#[test]
fn header_segment_type_error_names_expected_shape() {
    // A path segment that is neither a string nor a table trips the SegVisitor
    // `expecting` message; the surfaced error must name the accepted shapes.
    let text = "[[tasks.body.headers.rules]]\npath = [42]\naction = \"require\"\n";
    let err = parse_config(text).unwrap_err().to_string();
    assert!(
        err.contains("a string or"),
        "expected segment-shape hint, got: {err}"
    );
}

#[test]
fn header_rules_default_to_empty() {
    let config = parse_config("").unwrap();
    assert!(config.tasks.body.headers.rules.is_empty());
}

#[test]
fn parses_header_rule_name() {
    let config = parse_config(
        "[[tasks.body.headers.rules]]\nname = \"has-summary\"\npath = [\"Summary\"]\naction = \"require\"\n",
    )
    .unwrap();
    assert_eq!(
        config.tasks.body.headers.rules[0].name.as_deref(),
        Some("has-summary")
    );
}

#[test]
fn header_rule_name_defaults_to_none() {
    let config =
        parse_config("[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n")
            .unwrap();
    assert!(config.tasks.body.headers.rules[0].name.is_none());
}

#[test]
fn header_rule_label_prefers_name_over_position() {
    let config = parse_config(
        "[[tasks.body.headers.rules]]\nname = \"has-summary\"\npath = [\"Summary\"]\naction = \"require\"\n",
    )
    .unwrap();
    assert_eq!(
        config.tasks.body.headers.rules[0].label(3),
        "body.headers.rules.has-summary"
    );
}

#[test]
fn header_rule_label_falls_back_to_position() {
    let config =
        parse_config("[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n")
            .unwrap();
    assert_eq!(
        config.tasks.body.headers.rules[0].label(3),
        "body.headers.rules[3]"
    );
}

#[test]
fn rejects_duplicate_header_rule_names() {
    let text = "[[tasks.body.headers.rules]]\nname = \"dup\"\npath = [\"A\"]\naction = \"require\"\n\n\
                [[tasks.body.headers.rules]]\nname = \"dup\"\npath = [\"B\"]\naction = \"require\"\n";
    let err = parse_config(text).unwrap().validate().unwrap_err();
    assert!(
        err.contains("tasks.body.headers.rules[1]") && err.contains("dup"),
        "{err}"
    );
}
