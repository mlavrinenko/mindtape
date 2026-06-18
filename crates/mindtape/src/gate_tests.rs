#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use crate::config::{HeaderAction, HeaderPathSegment, HeaderRuleConfig};
use crate::model::Task;

use super::{GateError, check_headers, compile_rules};

fn bare_task(id: &str) -> Task {
    Task {
        id: id.to_owned(),
        file: format!("tasks/{id}.typ").into(),
        title: id.to_owned(),
        status: None,
        priority: None,
        grade: None,
        kind: None,
        estimate: BTreeMap::new(),
        depends_on: Vec::new(),
        relates_to: Vec::new(),
        supersedes: Vec::new(),
        duplicates: Vec::new(),
        tags: Vec::new(),
        due: None,
        start: None,
        extras: BTreeMap::new(),
    }
}

fn require(heading: &str) -> HeaderRuleConfig {
    HeaderRuleConfig {
        path: vec![HeaderPathSegment::Exact(heading.to_owned())],
        action: HeaderAction::Require,
        when: None,
        message: None,
    }
}

fn forbid(heading: &str) -> HeaderRuleConfig {
    HeaderRuleConfig {
        path: vec![HeaderPathSegment::Exact(heading.to_owned())],
        action: HeaderAction::Forbid,
        when: None,
        message: None,
    }
}

#[test]
fn compile_empty_rules_succeeds() {
    assert!(compile_rules(&[]).unwrap().is_empty());
}

#[test]
fn compile_rejects_empty_path() {
    let rule = HeaderRuleConfig {
        path: vec![],
        action: HeaderAction::Require,
        when: None,
        message: None,
    };
    assert!(matches!(compile_rules(&[rule]), Err(GateError::EmptyPath)));
}

#[test]
fn compile_rejects_invalid_regex() {
    let rule = HeaderRuleConfig {
        path: vec![HeaderPathSegment::Pattern {
            pattern: "[invalid".to_owned(),
            recursive: false,
        }],
        action: HeaderAction::Forbid,
        when: None,
        message: None,
    };
    assert!(matches!(
        compile_rules(&[rule]),
        Err(GateError::Regex { .. })
    ));
}

#[test]
fn compile_rejects_invalid_when() {
    let rule = HeaderRuleConfig {
        path: vec![HeaderPathSegment::Exact("Summary".to_owned())],
        action: HeaderAction::Require,
        when: Some("(".to_owned()), // unclosed parenthesis → parse error
        message: None,
    };
    assert!(matches!(compile_rules(&[rule]), Err(GateError::Filter(_))));
}

#[test]
fn require_present_heading_is_clean() {
    let task = bare_task("a");
    let rules = compile_rules(&[require("Summary")]).unwrap();
    assert!(check_headers(&task, "= Summary\n\nsome text\n", &rules).is_empty());
}

#[test]
fn require_absent_heading_is_violation() {
    let task = bare_task("a");
    let rules = compile_rules(&[require("Summary")]).unwrap();
    let violations = check_headers(&task, "= Introduction\n", &rules);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].field, "body.headers");
    assert!(violations[0].location.is_none());
}

#[test]
fn forbid_absent_heading_is_clean() {
    let task = bare_task("a");
    let rules = compile_rules(&[forbid("Phase")]).unwrap();
    assert!(check_headers(&task, "= Summary\n", &rules).is_empty());
}

#[test]
fn forbid_present_heading_carries_location() {
    let task = bare_task("a");
    let rules = compile_rules(&[forbid("Phase")]).unwrap();
    let violations = check_headers(&task, "= Phase\n", &rules);
    assert_eq!(violations.len(), 1);
    let loc = violations[0].location.as_ref().unwrap();
    assert_eq!(loc.line, 1);
}

#[test]
fn when_skips_non_matching_task() {
    let task = bare_task("a"); // no tags
    let rule = HeaderRuleConfig {
        path: vec![HeaderPathSegment::Exact("Summary".to_owned())],
        action: HeaderAction::Require,
        when: Some("has_tag(\"spec\")".to_owned()),
        message: None,
    };
    let rules = compile_rules(&[rule]).unwrap();
    // heading absent but `when` does not match → no violation
    assert!(check_headers(&task, "= Introduction\n", &rules).is_empty());
}

#[test]
fn when_applies_to_matching_task() {
    let mut task = bare_task("a");
    task.tags = vec!["spec".to_owned()];
    let rule = HeaderRuleConfig {
        path: vec![HeaderPathSegment::Exact("Summary".to_owned())],
        action: HeaderAction::Require,
        when: Some("has_tag(\"spec\")".to_owned()),
        message: None,
    };
    let rules = compile_rules(&[rule]).unwrap();
    assert_eq!(check_headers(&task, "= Introduction\n", &rules).len(), 1);
}

#[test]
fn multi_segment_path_direct_child() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        path: vec![
            HeaderPathSegment::Exact("Approach".to_owned()),
            HeaderPathSegment::Pattern {
                pattern: "^Phase".to_owned(),
                recursive: false,
            },
        ],
        action: HeaderAction::Forbid,
        when: None,
        message: None,
    };
    let rules = compile_rules(&[rule]).unwrap();
    let violations = check_headers(&task, "= Approach\n== Phase 1\n== Other\n", &rules);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("Phase 1"));
}

#[test]
fn recursive_segment_finds_deep_descendants() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        path: vec![
            HeaderPathSegment::Exact("Approach".to_owned()),
            HeaderPathSegment::Pattern {
                pattern: "^Phase".to_owned(),
                recursive: true,
            },
        ],
        action: HeaderAction::Forbid,
        when: None,
        message: None,
    };
    let rules = compile_rules(&[rule]).unwrap();
    let violations = check_headers(&task, "= Approach\n== Design\n=== Phase 1\n", &rules);
    assert_eq!(violations.len(), 1);
}

#[test]
fn direct_child_does_not_reach_deep_descendants() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        path: vec![
            HeaderPathSegment::Exact("Approach".to_owned()),
            HeaderPathSegment::Pattern {
                pattern: "^Phase".to_owned(),
                recursive: false,
            },
        ],
        action: HeaderAction::Forbid,
        when: None,
        message: None,
    };
    let rules = compile_rules(&[rule]).unwrap();
    // "Phase 1" is at depth 3 under Approach (not a direct child)
    assert!(check_headers(&task, "= Approach\n== Design\n=== Phase 1\n", &rules).is_empty());
}

#[test]
fn custom_message_is_used() {
    let task = bare_task("a");
    let rule = HeaderRuleConfig {
        path: vec![HeaderPathSegment::Exact("Summary".to_owned())],
        action: HeaderAction::Require,
        when: None,
        message: Some("must have Summary".to_owned()),
    };
    let rules = compile_rules(&[rule]).unwrap();
    let violations = check_headers(&task, "= Introduction\n", &rules);
    assert_eq!(violations[0].message, "must have Summary");
}

#[test]
fn no_rules_no_violations() {
    let task = bare_task("a");
    let rules = compile_rules(&[]).unwrap();
    assert!(check_headers(&task, "= Summary\n", &rules).is_empty());
}
