#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{Action, Check, Diagnostic, Matcher, Node, Rule, RuleError, Severity, run_rules};

fn leaf(text: &str, line: usize) -> Node {
    Node {
        kind: "heading".to_owned(),
        text: text.to_owned(),
        line,
        children: vec![],
    }
}

fn parent(text: &str, line: usize, children: Vec<Node>) -> Node {
    Node {
        kind: "heading".to_owned(),
        text: text.to_owned(),
        line,
        children,
    }
}

// --- require ---

#[test]
fn require_present_clean() {
    let roots = vec![leaf("Summary", 1)];
    let rule = Rule::path(vec![Matcher::exact("Summary")], Action::Require).unwrap();
    assert!(run_rules(&roots, &[rule]).is_empty());
}

#[test]
fn require_absent_one_diagnostic_line_none() {
    let roots = vec![leaf("Introduction", 1)];
    let rule = Rule::path(vec![Matcher::exact("Summary")], Action::Require).unwrap();
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].line.is_none());
    assert!(diags[0].message.contains("Summary"));
}

// --- forbid ---

#[test]
fn forbid_absent_clean() {
    let roots = vec![leaf("Summary", 1)];
    let rule = Rule::path(vec![Matcher::exact("Phase")], Action::Forbid).unwrap();
    assert!(run_rules(&roots, &[rule]).is_empty());
}

#[test]
fn forbid_present_line_set() {
    let roots = vec![leaf("Phase", 3)];
    let rule = Rule::path(vec![Matcher::exact("Phase")], Action::Forbid).unwrap();
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].line, Some(3));
}

// --- multi-segment direct child ---

#[test]
fn multi_segment_direct_child() {
    let roots = vec![parent(
        "Approach",
        1,
        vec![leaf("Phase 1", 2), leaf("Other", 3)],
    )];
    let rule = Rule::path(
        vec![
            Matcher::exact("Approach"),
            Matcher::pattern("^Phase", false).unwrap(),
        ],
        Action::Forbid,
    )
    .unwrap();
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].line, Some(2));
}

// --- recursive descendant ---

#[test]
fn recursive_descendant_match() {
    let roots = vec![parent(
        "Approach",
        1,
        vec![parent("Design", 2, vec![leaf("Phase 1", 3)])],
    )];
    let rule = Rule::path(
        vec![
            Matcher::exact("Approach"),
            Matcher::pattern("^Phase", true).unwrap(),
        ],
        Action::Forbid,
    )
    .unwrap();
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags.len(), 1);
}

// --- direct child does NOT reach deep descendants ---

#[test]
fn direct_child_does_not_reach_deep_descendants() {
    let roots = vec![parent(
        "Approach",
        1,
        vec![parent("Design", 2, vec![leaf("Phase 1", 3)])],
    )];
    let rule = Rule::path(
        vec![
            Matcher::exact("Approach"),
            Matcher::pattern("^Phase", false).unwrap(),
        ],
        Action::Forbid,
    )
    .unwrap();
    assert!(run_rules(&roots, &[rule]).is_empty());
}

// --- custom message ---

#[test]
fn custom_message_require() {
    let roots = vec![leaf("Introduction", 1)];
    let rule = Rule::path(vec![Matcher::exact("Summary")], Action::Require)
        .unwrap()
        .with_message("must have Summary");
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags[0].message, "must have Summary");
}

#[test]
fn custom_message_forbid() {
    let roots = vec![leaf("Phase", 2)];
    let rule = Rule::path(vec![Matcher::exact("Phase")], Action::Forbid)
        .unwrap()
        .with_message("no phases allowed");
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags[0].message, "no phases allowed");
}

// --- Rule::new errors ---

#[test]
fn rule_new_empty_path_returns_error() {
    assert_eq!(
        Rule::path(vec![], Action::Require).unwrap_err(),
        RuleError::EmptyPath
    );
}

// --- Matcher::pattern errors ---

#[test]
fn matcher_pattern_bad_regex_returns_error() {
    let err = Matcher::pattern("[invalid", false).unwrap_err();
    assert!(matches!(err, RuleError::Regex { .. }));
}

// --- describe_path formatting ---

#[test]
fn describe_path_exact() {
    let rule = Rule::path(vec![Matcher::exact("Summary")], Action::Require).unwrap();
    let roots: Vec<Node> = vec![];
    let diags = run_rules(&roots, &[rule]);
    assert!(diags[0].message.contains("'Summary'"));
}

#[test]
fn describe_path_pattern_non_recursive() {
    let rule = Rule::path(
        vec![Matcher::pattern("^Phase", false).unwrap()],
        Action::Require,
    )
    .unwrap();
    let diags = run_rules(&[], &[rule]);
    assert!(diags[0].message.contains("/^Phase/"));
    assert!(!diags[0].message.contains("/^Phase/.."));
}

#[test]
fn describe_path_pattern_recursive() {
    let rule = Rule::path(
        vec![Matcher::pattern("^Phase", true).unwrap()],
        Action::Require,
    )
    .unwrap();
    let diags = run_rules(&[], &[rule]);
    assert!(diags[0].message.contains("/^Phase/.."));
}

// --- builder methods ---

#[test]
fn rule_with_id_and_severity() {
    let rule = Rule::path(vec![Matcher::exact("X")], Action::Require)
        .unwrap()
        .with_id("rule-01")
        .with_severity(Severity::Warning);
    assert_eq!(rule.id.as_deref(), Some("rule-01"));
    assert_eq!(rule.severity, Severity::Warning);
}

// --- severity default ---

#[test]
fn severity_default_is_error() {
    assert_eq!(Severity::default(), Severity::Error);
}

// --- multi-segment path diagnostics ---

#[test]
fn multi_segment_path_in_diagnostic() {
    let rule = Rule::path(
        vec![Matcher::exact("A"), Matcher::exact("B")],
        Action::Require,
    )
    .unwrap();
    let diags = run_rules(&[], &[rule]);
    assert!(diags[0].message.contains("'A' > 'B'"));
}

// --- rule_id propagates ---

#[test]
fn rule_id_propagates_to_diagnostic() {
    let rule = Rule::path(vec![Matcher::exact("Missing")], Action::Require)
        .unwrap()
        .with_id("r1");
    let diags = run_rules(&[], &[rule]);
    assert_eq!(diags[0].rule_id.as_deref(), Some("r1"));
}

// --- forbid default message contains text ---

#[test]
fn forbid_default_message_contains_node_text() {
    let roots = vec![leaf("Phase 1", 5)];
    let rule = Rule::path(vec![Matcher::exact("Phase 1")], Action::Forbid).unwrap();
    let diags = run_rules(&roots, &[rule]);
    assert!(diags[0].message.contains("Phase 1"));
}

// --- diagnostic fields ---

#[test]
fn diagnostic_severity_propagates() {
    let roots = vec![leaf("X", 1)];
    let rule = Rule::path(vec![Matcher::exact("X")], Action::Forbid)
        .unwrap()
        .with_severity(Severity::Warning);
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags[0].severity, Severity::Warning);
}

// --- multiple matches with forbid ---

#[test]
fn forbid_emits_one_diagnostic_per_match() {
    let roots = vec![leaf("Phase", 1), leaf("Phase", 4)];
    let rule = Rule::path(vec![Matcher::exact("Phase")], Action::Forbid).unwrap();
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags.len(), 2);
}

// --- Diagnostic fields completeness ---

#[test]
fn diagnostic_clone_and_eq() {
    let d = Diagnostic {
        rule_id: Some("x".to_owned()),
        severity: Severity::Error,
        line: Some(1),
        message: "msg".to_owned(),
    };
    assert_eq!(d.clone(), d);
}

// --- unique cardinality ---

#[test]
fn unique_exactly_one_clean() {
    let roots = vec![leaf("Summary", 1)];
    let rule = Rule::path(vec![Matcher::exact("Summary")], Action::Unique).unwrap();
    assert!(run_rules(&roots, &[rule]).is_empty());
}

#[test]
fn unique_absent_is_missing_line_none() {
    let roots = vec![leaf("Intro", 1)];
    let rule = Rule::path(vec![Matcher::exact("Summary")], Action::Unique).unwrap();
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].line.is_none());
    assert!(diags[0].message.contains("not found"));
}

#[test]
fn unique_duplicates_flag_extras_at_their_lines() {
    let roots = vec![leaf("Summary", 1), leaf("Summary", 5), leaf("Summary", 9)];
    let rule = Rule::path(vec![Matcher::exact("Summary")], Action::Unique).unwrap();
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags.len(), 2);
    assert_eq!(diags[0].line, Some(5));
    assert_eq!(diags[1].line, Some(9));
    assert!(diags[0].message.contains("duplicate"));
}

#[test]
fn unique_keeps_earliest_as_canonical() {
    // out-of-order input: first occurrence (by line) is the survivor
    let roots = vec![leaf("Summary", 9), leaf("Summary", 2)];
    let rule = Rule::path(vec![Matcher::exact("Summary")], Action::Unique).unwrap();
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].line, Some(9));
}

// --- ordering ---

#[test]
fn order_in_order_clean() {
    let roots = vec![leaf("Summary", 1), leaf("Scope", 5)];
    let rule = Rule::order(
        vec![Matcher::exact("Summary")],
        vec![Matcher::exact("Scope")],
    )
    .unwrap();
    assert!(run_rules(&roots, &[rule]).is_empty());
}

#[test]
fn order_reversed_is_violation_at_second() {
    let roots = vec![leaf("Scope", 1), leaf("Summary", 5)];
    let rule = Rule::order(
        vec![Matcher::exact("Summary")],
        vec![Matcher::exact("Scope")],
    )
    .unwrap();
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].line, Some(1)); // earliest offending second
    assert!(diags[0].message.contains("must precede"));
}

#[test]
fn order_missing_side_is_not_a_violation() {
    let roots = vec![leaf("Summary", 1)]; // no Scope
    let rule = Rule::order(
        vec![Matcher::exact("Summary")],
        vec![Matcher::exact("Scope")],
    )
    .unwrap();
    assert!(run_rules(&roots, &[rule]).is_empty());
}

#[test]
fn order_strict_flags_interleaving() {
    // a second appears before the last first → out of order
    let roots = vec![leaf("Summary", 1), leaf("Scope", 3), leaf("Summary", 7)];
    let rule = Rule::order(
        vec![Matcher::exact("Summary")],
        vec![Matcher::exact("Scope")],
    )
    .unwrap();
    assert_eq!(run_rules(&roots, &[rule]).len(), 1);
}

#[test]
fn order_empty_path_errors() {
    assert_eq!(
        Rule::order(vec![], vec![Matcher::exact("X")]).unwrap_err(),
        RuleError::EmptyPath
    );
    assert_eq!(
        Rule::order(vec![Matcher::exact("X")], vec![]).unwrap_err(),
        RuleError::EmptyPath
    );
}

// --- closed-world allowlist ---

#[test]
fn allow_root_permits_listed_flags_others() {
    let roots = vec![leaf("Context", 1), leaf("Choice", 3), leaf("Random", 5)];
    let rule = Rule::allow(
        None,
        vec![Matcher::exact("Context"), Matcher::exact("Choice")],
    )
    .unwrap();
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].line, Some(5));
    assert!(diags[0].message.contains("Random"));
}

#[test]
fn allow_root_all_listed_clean() {
    let roots = vec![leaf("Context", 1), leaf("Choice", 3)];
    let rule = Rule::allow(
        None,
        vec![Matcher::exact("Context"), Matcher::exact("Choice")],
    )
    .unwrap();
    assert!(run_rules(&roots, &[rule]).is_empty());
}

#[test]
fn allow_scoped_constrains_only_children() {
    // top-level "Other" is untouched; only Decision's children are constrained
    let roots = vec![
        parent("Decision", 1, vec![leaf("Context", 2), leaf("Stray", 3)]),
        leaf("Other", 9),
    ];
    let rule = Rule::allow(
        Some(vec![Matcher::exact("Decision")]),
        vec![Matcher::exact("Context")],
    )
    .unwrap();
    let diags = run_rules(&roots, &[rule]);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].line, Some(3));
    assert!(diags[0].message.contains("Stray"));
}

#[test]
fn allow_pattern_matcher_permits() {
    let roots = vec![leaf("Phase 1", 1), leaf("Phase 2", 2)];
    let rule = Rule::allow(None, vec![Matcher::pattern("^Phase", false).unwrap()]).unwrap();
    assert!(run_rules(&roots, &[rule]).is_empty());
}

#[test]
fn allow_empty_allowed_errors() {
    assert_eq!(Rule::allow(None, vec![]).unwrap_err(), RuleError::EmptyPath);
}

#[test]
fn allow_empty_scope_errors() {
    assert_eq!(
        Rule::allow(Some(vec![]), vec![Matcher::exact("X")]).unwrap_err(),
        RuleError::EmptyPath
    );
}

// --- Check is inspectable ---

#[test]
fn rule_carries_check() {
    let rule = Rule::path(vec![Matcher::exact("X")], Action::Forbid).unwrap();
    assert!(matches!(rule.check, Check::Path { .. }));
}
