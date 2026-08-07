#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::{Facet, LsRule, parse_import_view, validate_ls_rules};
use crate::config::Config;
use crate::config_views::ViewSpec;
use crate::parse_config;

fn view(filter: &str) -> ViewSpec {
    ViewSpec {
        filter: filter.to_owned(),
        ..ViewSpec::default()
    }
}

/// Validate a single rule against `views`, for a compact per-field test.
fn problems_with_views(rule: &LsRule, views: &BTreeMap<String, ViewSpec>) -> Vec<String> {
    validate_ls_rules(std::slice::from_ref(rule), views)
}

/// Validate a single rule with no declared views.
fn problems(rule: &LsRule) -> Vec<String> {
    problems_with_views(rule, &BTreeMap::new())
}

fn assert_has(problems: &[String], needle: &str) {
    assert!(problems.iter().any(|p| p.contains(needle)), "{problems:?}");
}

#[test]
fn bare_name_selects_all_facets() {
    let parsed = parse_import_view("inbox").unwrap();
    assert_eq!(parsed.view, "inbox");
    assert_eq!(
        parsed.facets,
        vec![Facet::Filter, Facet::Sort, Facet::Columns]
    );
}

#[test]
fn parenthesised_subset_selects_named_facets() {
    let parsed = parse_import_view("release(sort, columns)").unwrap();
    assert_eq!(parsed.view, "release");
    assert_eq!(parsed.facets, vec![Facet::Sort, Facet::Columns]);
}

#[test]
fn single_facet_selector_parses() {
    let parsed = parse_import_view("release(sort)").unwrap();
    assert_eq!(parsed.facets, vec![Facet::Sort]);
}

#[test]
fn empty_parens_is_an_error() {
    assert!(parse_import_view("release()").is_err());
}

#[test]
fn empty_entry_is_an_error() {
    assert!(parse_import_view("").is_err());
    assert!(parse_import_view("   ").is_err());
}

#[test]
fn missing_view_name_is_an_error() {
    assert!(parse_import_view("(sort)").is_err());
}

#[test]
fn missing_closing_paren_is_an_error() {
    assert!(parse_import_view("release(sort").is_err());
}

#[test]
fn unknown_facet_is_an_error() {
    assert!(parse_import_view("release(bogus)").is_err());
}

#[test]
fn whitespace_around_facets_is_trimmed() {
    let parsed = parse_import_view(" release ( sort , columns ) ").unwrap();
    assert_eq!(parsed.view, "release");
    assert_eq!(parsed.facets, vec![Facet::Sort, Facet::Columns]);
}

#[test]
fn validate_rules_accepts_well_formed_rules() {
    let views = BTreeMap::from([
        ("inbox".to_owned(), view("")),
        ("release".to_owned(), view("")),
    ]);
    let imports = LsRule {
        name: Some("release".to_owned()),
        condition: Some(r#"on_branch("release/*")"#.to_owned()),
        import_views: vec!["inbox".to_owned()],
        ..LsRule::default()
    };
    assert!(problems_with_views(&imports, &views).is_empty());

    let spliced = LsRule {
        sort: vec!["@release".to_owned()],
        columns: vec!["@release".to_owned()],
        ..LsRule::default()
    };
    assert!(problems_with_views(&spliced, &views).is_empty());
}

fn rule_condition(expr: &str) -> LsRule {
    LsRule {
        condition: Some(expr.to_owned()),
        ..LsRule::default()
    }
}

fn rule_filter(expr: &str) -> LsRule {
    LsRule {
        filter: Some(expr.to_owned()),
        ..LsRule::default()
    }
}

/// Build a rule from a list-valued field (`import-views` / `sort` /
/// `columns`): `build` places the owned entries on the right field.
fn rule_with_list(entries: &[&str], build: impl Fn(Vec<String>) -> LsRule) -> LsRule {
    build(entries.iter().map(|&s| s.to_owned()).collect())
}

/// Each case is one rule with exactly one bad field, plus a substring the
/// resulting problem must contain — covers every field `validate_ls_rules`
/// checks without repeating the same rule-then-assert skeleton per field.
#[test]
fn validate_rules_reports_one_problem_per_bad_field() {
    let import_views = |v| LsRule {
        import_views: v,
        ..LsRule::default()
    };
    let sort = |v| LsRule {
        sort: v,
        ..LsRule::default()
    };
    let columns = |v| LsRule {
        columns: v,
        ..LsRule::default()
    };
    let cases: Vec<(LsRule, &str)> = vec![
        (rule_condition("priority > 5"), "ls.rules[0].condition"),
        (
            rule_with_list(&["ghost"], import_views),
            "unknown view `ghost`",
        ),
        (rule_with_list(&["broken("], import_views), "import-views"),
        (rule_filter("!!bad!!"), ".filter:"),
        (rule_filter("@ghost"), "unknown view reference @ghost"),
        (rule_with_list(&["@ghost"], sort), ".sort:"),
        (rule_with_list(&["no_such_field"], sort), ".sort:"),
        (rule_with_list(&["bogus"], columns), ".columns:"),
    ];
    for (rule, needle) in cases {
        assert_has(&problems(&rule), needle);
    }
}

#[test]
fn validate_rules_prefixes_each_problem_by_index() {
    let rules = [
        LsRule::default(),
        LsRule {
            columns: vec!["bogus".to_owned()],
            ..LsRule::default()
        },
    ];
    let problems = validate_ls_rules(&rules, &BTreeMap::new());
    assert_has(&problems, "ls.rules[1]");
}

#[test]
fn contains_fields_defaults_to_empty_when_unset() {
    // Empty is the signal `filter_predicates::default_contains_fields` reads
    // to fall back to the shipped `title`/`id`/`body` sweep.
    assert!(Config::default().ls.contains_fields.is_empty());
    assert!(parse_config("").unwrap().ls.contains_fields.is_empty());
}

#[test]
fn contains_fields_accepts_the_full_vocabulary_including_note_and_tags() {
    let cfg =
        parse_config("[ls]\ncontains-fields = [\"title\", \"id\", \"body\", \"note\", \"tags\"]\n")
            .unwrap();
    assert_eq!(
        cfg.ls.contains_fields,
        vec!["title", "id", "body", "note", "tags"]
    );
}

#[test]
fn contains_fields_accepts_a_narrowed_list() {
    let cfg = parse_config("[ls]\ncontains-fields = [\"title\"]\n").unwrap();
    assert_eq!(cfg.ls.contains_fields, vec!["title"]);
}

#[test]
fn unknown_contains_field_is_rejected_at_load() {
    let err = parse_config("[ls]\ncontains-fields = [\"bogus\"]\n").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown field `bogus`"), "{msg}");
    // The valid set is listed, not just named as invalid.
    assert!(msg.contains("title"), "{msg}");
    assert!(msg.contains("note"), "{msg}");
    assert!(msg.contains("tags"), "{msg}");
}
