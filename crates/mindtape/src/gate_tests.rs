#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::config::{Config, HeaderAction, HeaderPathSegment, HeaderRuleConfig};
use crate::test_support::bare_task;

use super::{GateError, check_headers, compile_rules};

fn ex(text: &str) -> HeaderPathSegment {
    HeaderPathSegment::Exact(text.to_owned())
}

fn rx(pattern: &str, recursive: bool) -> HeaderPathSegment {
    HeaderPathSegment::Pattern {
        pattern: pattern.to_owned(),
        recursive,
    }
}

/// A blank rule of the given action; fill the fields the action uses.
fn cfg(action: HeaderAction) -> HeaderRuleConfig {
    HeaderRuleConfig {
        name: None,
        action,
        path: vec![],
        before: vec![],
        after: vec![],
        scope: vec![],
        allowed: vec![],
        when: None,
        message: None,
    }
}

fn require(heading: &str) -> HeaderRuleConfig {
    HeaderRuleConfig {
        path: vec![ex(heading)],
        ..cfg(HeaderAction::Require)
    }
}

fn forbid(heading: &str) -> HeaderRuleConfig {
    HeaderRuleConfig {
        path: vec![ex(heading)],
        ..cfg(HeaderAction::Forbid)
    }
}

// --- compile errors ---

#[test]
fn compile_empty_rules_succeeds() {
    assert!(compile_rules(&[]).unwrap().is_empty());
}

#[test]
fn compile_rejects_empty_path() {
    let rule = cfg(HeaderAction::Require);
    assert!(matches!(compile_rules(&[rule]), Err(GateError::EmptyPath)));
}

#[test]
fn compile_rejects_invalid_regex() {
    let rule = HeaderRuleConfig {
        path: vec![rx("[invalid", false)],
        ..cfg(HeaderAction::Forbid)
    };
    assert!(matches!(
        compile_rules(&[rule]),
        Err(GateError::Regex { .. })
    ));
}

#[test]
fn compile_rejects_invalid_when() {
    let rule = HeaderRuleConfig {
        when: Some("(".to_owned()), // unclosed parenthesis → parse error
        ..require("Summary")
    };
    assert!(matches!(compile_rules(&[rule]), Err(GateError::Filter(_))));
}

#[test]
fn compile_rejects_unexpected_fields() {
    // a require rule that also names `allowed`
    let rule = HeaderRuleConfig {
        path: vec![ex("Summary")],
        allowed: vec![ex("X")],
        ..cfg(HeaderAction::Require)
    };
    let err = compile_rules(&[rule]).unwrap_err();
    assert!(matches!(err, GateError::UnexpectedFields { .. }));
    assert!(err.to_string().contains("allowed"));
}

// --- cardinality: require / forbid ---

#[test]
fn require_present_heading_is_clean() {
    let task = bare_task("a");
    let rules = compile_rules(&[require("Summary")]).unwrap();
    assert!(
        check_headers(
            &task,
            "= Summary\n\nsome text\n",
            &rules,
            &Config::default()
        )
        .is_empty()
    );
}

#[test]
fn require_absent_heading_is_violation() {
    let task = bare_task("a");
    let rules = compile_rules(&[require("Summary")]).unwrap();
    let violations = check_headers(&task, "= Introduction\n", &rules, &Config::default());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].field, "body.headers.rules[0]");
}

#[test]
fn missing_section_reports_at_the_task_with_line() {
    let mut task = bare_task("a");
    task.marker_line = 7;
    let rules = compile_rules(&[require("Summary")]).unwrap();
    let violations = check_headers(&task, "= Introduction\n", &rules, &Config::default());
    let loc = violations[0].location.as_ref().unwrap();
    assert_eq!(loc.line, 7);
    assert_eq!(loc.file, task.file);
}

#[test]
fn named_rule_reports_its_name_instead_of_position() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        name: Some("has-summary".to_owned()),
        ..require("Summary")
    };
    let rules = compile_rules(&[rule]).unwrap();
    let violations = check_headers(&task, "= Introduction\n", &rules, &Config::default());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].field, "body.headers.rules.has-summary");
}

#[test]
fn require_allows_duplicates() {
    let task = bare_task("a");
    let rules = compile_rules(&[require("Summary")]).unwrap();
    assert!(check_headers(&task, "= Summary\n= Summary\n", &rules, &Config::default()).is_empty());
}

#[test]
fn forbid_absent_heading_is_clean() {
    let task = bare_task("a");
    let rules = compile_rules(&[forbid("Phase")]).unwrap();
    assert!(check_headers(&task, "= Summary\n", &rules, &Config::default()).is_empty());
}

#[test]
fn forbid_present_heading_carries_location() {
    let task = bare_task("a");
    let rules = compile_rules(&[forbid("Phase")]).unwrap();
    let violations = check_headers(&task, "= Phase\n", &rules, &Config::default());
    assert_eq!(violations.len(), 1);
    let loc = violations[0].location.as_ref().unwrap();
    assert_eq!(loc.line, 1);
}

// --- cardinality: unique ---

#[test]
fn unique_single_is_clean() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        path: vec![ex("Summary")],
        ..cfg(HeaderAction::Unique)
    };
    let rules = compile_rules(&[rule]).unwrap();
    assert!(check_headers(&task, "= Summary\n= Scope\n", &rules, &Config::default()).is_empty());
}

#[test]
fn unique_duplicate_flags_extra() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        path: vec![ex("Summary")],
        ..cfg(HeaderAction::Unique)
    };
    let rules = compile_rules(&[rule]).unwrap();
    let violations = check_headers(&task, "= Summary\n= Summary\n", &rules, &Config::default());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].location.as_ref().unwrap().line, 2);
    assert!(violations[0].message.contains("duplicate"));
}

// --- ordering ---

#[test]
fn order_in_order_is_clean() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        before: vec![ex("Summary")],
        after: vec![ex("Scope")],
        ..cfg(HeaderAction::Order)
    };
    let rules = compile_rules(&[rule]).unwrap();
    assert!(check_headers(&task, "= Summary\n= Scope\n", &rules, &Config::default()).is_empty());
}

#[test]
fn order_reversed_is_violation() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        before: vec![ex("Summary")],
        after: vec![ex("Scope")],
        ..cfg(HeaderAction::Order)
    };
    let rules = compile_rules(&[rule]).unwrap();
    let violations = check_headers(&task, "= Scope\n= Summary\n", &rules, &Config::default());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].location.as_ref().unwrap().line, 1);
    assert!(violations[0].message.contains("must precede"));
}

#[test]
fn order_rejects_path_field() {
    let rule = HeaderRuleConfig {
        path: vec![ex("X")],
        before: vec![ex("Summary")],
        after: vec![ex("Scope")],
        ..cfg(HeaderAction::Order)
    };
    assert!(matches!(
        compile_rules(&[rule]),
        Err(GateError::UnexpectedFields { .. })
    ));
}

// --- closed-world allowlist ---

#[test]
fn allowlist_top_level_flags_stray() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        allowed: vec![ex("Context"), ex("Choice")],
        ..cfg(HeaderAction::Allowlist)
    };
    let rules = compile_rules(&[rule]).unwrap();
    let violations = check_headers(
        &task,
        "= Context\n= Choice\n= Random\n",
        &rules,
        &Config::default(),
    );
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].location.as_ref().unwrap().line, 3);
    assert!(violations[0].message.contains("Random"));
}

#[test]
fn allowlist_all_listed_is_clean() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        allowed: vec![ex("Context"), ex("Choice")],
        ..cfg(HeaderAction::Allowlist)
    };
    let rules = compile_rules(&[rule]).unwrap();
    assert!(check_headers(&task, "= Context\n= Choice\n", &rules, &Config::default()).is_empty());
}

#[test]
fn allowlist_scoped_constrains_children_only() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        scope: vec![ex("Decision")],
        allowed: vec![ex("Context")],
        ..cfg(HeaderAction::Allowlist)
    };
    let rules = compile_rules(&[rule]).unwrap();
    // top-level "Other" is untouched; "Stray" under Decision is flagged
    let violations = check_headers(
        &task,
        "= Decision\n== Context\n== Stray\n= Other\n",
        &rules,
        &Config::default(),
    );
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("Stray"));
}

// --- `when` gating ---

#[test]
fn when_skips_non_matching_task() {
    let task = bare_task("a"); // no tags
    let rule = HeaderRuleConfig {
        when: Some("has_tag(\"spec\")".to_owned()),
        ..require("Summary")
    };
    let rules = compile_rules(&[rule]).unwrap();
    // heading absent but `when` does not match → no violation
    assert!(check_headers(&task, "= Introduction\n", &rules, &Config::default()).is_empty());
}

#[test]
fn when_applies_to_matching_task() {
    let mut task = bare_task("a");
    task.tags = vec!["spec".to_owned()];
    let rule = HeaderRuleConfig {
        when: Some("has_tag(\"spec\")".to_owned()),
        ..require("Summary")
    };
    let rules = compile_rules(&[rule]).unwrap();
    assert_eq!(
        check_headers(&task, "= Introduction\n", &rules, &Config::default()).len(),
        1
    );
}

// --- multi-segment / recursive paths ---

#[test]
fn multi_segment_path_direct_child() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        path: vec![ex("Approach"), rx("^Phase", false)],
        ..cfg(HeaderAction::Forbid)
    };
    let rules = compile_rules(&[rule]).unwrap();
    let violations = check_headers(
        &task,
        "= Approach\n== Phase 1\n== Other\n",
        &rules,
        &Config::default(),
    );
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("Phase 1"));
}

#[test]
fn recursive_segment_finds_deep_descendants() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        path: vec![ex("Approach"), rx("^Phase", true)],
        ..cfg(HeaderAction::Forbid)
    };
    let rules = compile_rules(&[rule]).unwrap();
    let violations = check_headers(
        &task,
        "= Approach\n== Design\n=== Phase 1\n",
        &rules,
        &Config::default(),
    );
    assert_eq!(violations.len(), 1);
}

#[test]
fn direct_child_does_not_reach_deep_descendants() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        path: vec![ex("Approach"), rx("^Phase", false)],
        ..cfg(HeaderAction::Forbid)
    };
    let rules = compile_rules(&[rule]).unwrap();
    // "Phase 1" is at depth 3 under Approach (not a direct child)
    assert!(
        check_headers(
            &task,
            "= Approach\n== Design\n=== Phase 1\n",
            &rules,
            &Config::default()
        )
        .is_empty()
    );
}

// --- message / empty ---

#[test]
fn custom_message_is_used() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        message: Some("must have Summary".to_owned()),
        ..require("Summary")
    };
    let rules = compile_rules(&[rule]).unwrap();
    let violations = check_headers(&task, "= Introduction\n", &rules, &Config::default());
    assert_eq!(violations[0].message, "must have Summary");
}

#[test]
fn no_rules_no_violations() {
    let task = bare_task("a");
    let rules = compile_rules(&[]).unwrap();
    assert!(check_headers(&task, "= Summary\n", &rules, &Config::default()).is_empty());
}
