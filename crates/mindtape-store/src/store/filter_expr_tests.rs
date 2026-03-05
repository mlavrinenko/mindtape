#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn bare_done() {
    let frag = translate_expr("done").unwrap();
    assert_eq!(frag.condition, "t.is_done = 1");
    assert!(frag.params.is_empty());
    assert!(!frag.needs_fts_join);
}

#[test]
fn not_done() {
    let frag = translate_expr("!done").unwrap();
    assert_eq!(frag.condition, "NOT (t.is_done = 1)");
}

#[test]
fn done_eq_false() {
    let frag = translate_expr("done == false").unwrap();
    assert_eq!(frag.condition, "t.is_done = ?");
    assert_eq!(frag.params, vec!["0"]);
}

#[test]
fn due_lt() {
    let frag = translate_expr("due < \"2026-03-10\"").unwrap();
    assert_eq!(frag.condition, "(t.due IS NOT NULL AND t.due < ?)");
    assert_eq!(frag.params, vec!["2026-03-10"]);
}

#[test]
fn due_eq() {
    let frag = translate_expr("due == \"2026-03-10\"").unwrap();
    assert_eq!(frag.condition, "t.due = ?");
    assert_eq!(frag.params, vec!["2026-03-10"]);
}

#[test]
fn rank_geq() {
    let frag = translate_expr("rank >= 5").unwrap();
    assert_eq!(frag.condition, "(t.rank IS NOT NULL AND t.rank >= ?)");
    assert_eq!(frag.params, vec!["5"]);
}

#[test]
fn title_eq() {
    let frag = translate_expr("title == \"Deploy\"").unwrap();
    assert_eq!(frag.condition, "t.title = ?");
    assert_eq!(frag.params, vec!["Deploy"]);
}

#[test]
fn and_or_combined() {
    let frag = translate_expr(
        "due < \"2026-03-10\" && (has_tag(\"urgent\") || rank >= 5)",
    )
    .unwrap();
    assert!(frag.condition.contains("AND"));
    assert!(frag.condition.contains("OR"));
    assert!(!frag.needs_fts_join);
}

#[test]
fn has_due() {
    let frag = translate_expr("has(due)").unwrap();
    assert_eq!(frag.condition, "t.due IS NOT NULL");
    assert!(frag.params.is_empty());
}

#[test]
fn has_tag_presence() {
    let frag = translate_expr("has(tag)").unwrap();
    assert!(frag.condition.contains("EXISTS"));
    assert!(frag.params.is_empty());
}

#[test]
fn has_tag_specific() {
    let frag = translate_expr("has_tag(\"urgent\")").unwrap();
    assert!(frag.condition.contains("EXISTS"));
    assert!(frag.condition.contains("tp.value = ?"));
    assert_eq!(frag.params, vec!["urgent"]);
}

#[test]
fn search_sets_fts_flag() {
    let frag = translate_expr("search(\"deploy\")").unwrap();
    assert!(frag.needs_fts_join);
    assert!(frag.condition.contains("MATCH"));
    assert_eq!(frag.params, vec!["deploy"]);
}

#[test]
fn contains_title() {
    let frag = translate_expr("contains(title, \"deploy\")").unwrap();
    assert!(frag.condition.contains("LIKE"));
    assert!(frag.condition.contains("COLLATE NOCASE"));
    assert_eq!(frag.params, vec!["deploy"]);
}

#[test]
fn complex_expression() {
    let frag = translate_expr(
        "!done && (has(due) || has_tag(\"work\")) && rank >= 3",
    )
    .unwrap();
    assert!(frag.condition.contains("NOT"));
    assert!(frag.condition.contains("AND"));
    assert!(frag.condition.contains("OR"));
}

#[test]
fn unknown_field_errors() {
    let result = translate_expr("foobar == 1");
    assert!(result.is_err());
}

#[test]
fn invalid_syntax_errors() {
    let result = translate_expr("((( bad");
    assert!(result.is_err());
}

#[test]
fn unknown_function_errors() {
    let result = translate_expr("bogus(\"x\")");
    assert!(result.is_err());
}

#[test]
fn bare_non_boolean_field_errors() {
    let result = translate_expr("title");
    assert!(result.is_err());
}

#[test]
fn has_unknown_prop_errors() {
    let result = translate_expr("has(foobar)");
    assert!(result.is_err());
}
