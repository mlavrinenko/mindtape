//! Tests for `Applied`'s footer, supersession, and `bare_view_ref`'s `@name`
//! parsing. Cascade resolution (`resolve`/`resolve_bare`) lives in
//! `cmd/ls/rules_tests.rs`. Shares its `args`/`project`/`project_with_config`
//! fixtures.
#![allow(clippy::unwrap_used)]

use crate::cmd::ls::rules::tests::{args, project, project_with_config, resolve};

use super::{Applied, MATCH_ALL, empty_hint, footer};

/// An `Applied` whose facets came from `filter`, `sort` and `columns`, with
/// `rule` matched.
fn applied(rule: Option<&str>, filter: &str, sort: &str, columns: &str) -> Applied {
    Applied {
        rule: rule.map(str::to_owned),
        filter_from: filter.to_owned(),
        sort_from: sort.to_owned(),
        columns_from: columns.to_owned(),
        ..Applied::default()
    }
}

/// `applied` with the filter value and description a view or a rule supplied.
fn filtered(mut applied: Applied, value: &str, description: Option<&str>) -> Applied {
    applied.filter_value = value.to_owned();
    applied.filter_description = description.map(str::to_owned);
    applied
}

/// The default view's filter, the long one the footer wraps.
const DEFAULT_FILTER: &str = "!terminal && (miss(start) || start <= now())";

/// The default view's `Applied`, carrying the filter it applied.
fn default_view() -> Applied {
    filtered(
        applied(None, "default", "default", "default"),
        DEFAULT_FILTER,
        None,
    )
}

/// A config whose `agent` view carries a description, plus `extra` TOML.
fn agent_config(extra: &str) -> String {
    format!(
        r#"
[views.agent]
filter = 'assignee == "agent"'
description = "Open tasks an agent can take"
{extra}
"#
    )
}

/// The footer of a listing `config` resolves under `filter`.
fn resolved_footer(config: &str, filter: Option<&str>, shown: usize, total: usize) -> String {
    let (_dir, project) = project(config);
    let (_bundle, applied) = resolve(&project, &args(filter)).unwrap();
    footer(&applied, shown, total, 0, None)
}

/// Assert the footer of `applied` showing `shown` of `total` tasks, with every
/// file loaded.
fn assert_footer(applied: &Applied, shown: usize, total: usize, expected: &str) {
    assert_eq!(footer(applied, shown, total, 0, None), expected);
}

/// The default view's footer showing 2 of 1013 tasks, wrapped at `width`.
fn wrapped(width: usize) -> String {
    footer(&default_view(), 2, 1013, 0, Some(width))
}

#[test]
fn footer_names_the_view_and_counts_the_project() {
    let from_view = applied(None, "default", "default", "default");
    assert_footer(&from_view, 3, 12, "3 of 12 tasks · default view");
    assert_footer(&from_view, 1, 1, "1 of 1 task · default view");
    // A rule that set the filter is named once, not again for its sort.
    let from_rule = applied(Some("release"), "release", "release", "release");
    assert_footer(&from_rule, 0, 2, "0 of 2 tasks · release rule");
}

#[test]
fn footer_names_the_filter_a_view_applied() {
    assert_footer(
        &default_view(),
        2,
        1013,
        "2 of 1013 tasks · default view (!terminal && (miss(start) || start <= now()))",
    );
}

#[test]
fn footer_shows_a_view_description_in_place_of_its_filter() {
    let from_view = filtered(
        applied(None, "agent", "agent", "agent"),
        "assignee == \"agent\" && !terminal",
        Some("Open tasks an agent can take"),
    );
    assert_footer(
        &from_view,
        3,
        1013,
        "3 of 1013 tasks · agent view (Open tasks an agent can take)",
    );
}

#[test]
fn footer_names_the_filter_a_rule_applied() {
    let narrowing = r#"
[[ls.rules]]
name = "release"
filter = "priority > 5"
"#;
    assert_eq!(
        resolved_footer(narrowing, None, 0, 2),
        "0 of 2 tasks · release rule (priority > 5)"
    );
    // A rule's own filter replaces the description of the view it imported.
    let overriding = agent_config(
        "[[ls.rules]]\nname = \"release\"\nimport-views = [\"agent\"]\nfilter = \"priority > 5\"",
    );
    assert_eq!(
        resolved_footer(&overriding, None, 0, 2),
        "0 of 2 tasks · release rule (priority > 5)"
    );
}

#[test]
fn footer_names_a_sort_rule_after_the_view() {
    let shared = applied(Some("release"), "inbox", "release", "inbox");
    assert_footer(&shared, 3, 12, "3 of 12 tasks · inbox view · release sort");
    let sort_only = applied(Some("release"), MATCH_ALL, "release", MATCH_ALL);
    assert_footer(&sort_only, 0, 2, "0 of 2 tasks · release sort");
}

#[test]
fn footer_says_your_filter_whatever_else_the_user_typed() {
    let typed =
        |sort: &str, columns: &str| footer(&applied(None, "manual", sort, columns), 2, 2, 0, None);
    assert_eq!(typed(MATCH_ALL, MATCH_ALL), "2 of 2 tasks · your filter");
    assert_eq!(typed("manual", MATCH_ALL), "2 of 2 tasks · your filter");
    assert_eq!(typed("manual", "manual"), "2 of 2 tasks · your filter");
}

#[test]
fn footer_keeps_your_filter_bare() {
    let typed = filtered(
        applied(None, "manual", MATCH_ALL, MATCH_ALL),
        "priority > 5",
        None,
    );
    assert_footer(&typed, 2, 2, "2 of 2 tasks · your filter");
}

#[test]
fn footer_keeps_your_filter_bare_over_a_views_description() {
    let config = agent_config("[[ls.rules]]\nname = \"release\"\nimport-views = [\"agent\"]");
    assert_eq!(
        resolved_footer(&config, Some("priority > 5"), 2, 2),
        "2 of 2 tasks · your filter"
    );
}

#[test]
fn footer_leaves_out_a_view_that_only_sorts_or_picks_columns() {
    let narrowed = applied(None, "manual", "default", "default");
    assert_footer(&narrowed, 2, 5, "2 of 5 tasks · your filter");
}

#[test]
fn footer_names_a_rule_that_only_picks_columns() {
    let columns_only = applied(Some("wide"), MATCH_ALL, MATCH_ALL, "wide");
    assert_footer(&columns_only, 2, 2, "2 of 2 tasks · wide rule");
}

#[test]
fn footer_is_the_count_alone_when_nothing_shaped_the_listing() {
    assert_footer(&Applied::match_all(), 2, 2, "2 of 2 tasks");
}

#[test]
fn footer_leaves_a_view_with_no_filter_to_its_name() {
    let empty = filtered(applied(None, "inbox", MATCH_ALL, MATCH_ALL), "", None);
    assert_footer(&empty, 2, 2, "2 of 2 tasks · inbox view");
}

#[test]
fn footer_counts_the_broken_files_last() {
    assert_eq!(
        footer(&default_view(), 1, 2, 1, None),
        "1 of 2 tasks · default view (!terminal && (miss(start) || start <= now())) · 1 broken"
    );
    assert_eq!(
        footer(&Applied::match_all(), 0, 0, 3, None),
        "0 of 0 tasks · 3 broken"
    );
}

#[test]
fn footer_leaves_an_overridden_rule_to_the_json() {
    let applied = Applied {
        rule_superseded: true,
        ..applied(Some("release"), "manual", MATCH_ALL, MATCH_ALL)
    };
    assert_footer(&applied, 1, 4, "1 of 4 tasks · your filter");
}

#[test]
fn footer_wraps_at_the_terminal_width() {
    let cases = [
        // Wide enough for the whole line.
        (
            80,
            "2 of 1013 tasks · default view (!terminal && (miss(start) || start <= now()))",
        ),
        // The filter fits a line of its own, so the wrap lands on a separator
        // and the count keeps the line the table ends on.
        (
            70,
            "2 of 1013 tasks\n  · default view (!terminal && (miss(start) || start <= now()))",
        ),
        // The filter no longer fits, so it breaks at a space. The count is the
        // shortest part and stays whole.
        (
            60,
            "2 of 1013 tasks · default view (!terminal && (miss(start) ||\n  start <= now()))",
        ),
        // Narrow enough that the filter needs two continuation lines.
        (
            40,
            "2 of 1013 tasks · default view\n  (!terminal && (miss(start) || start <=\n  now()))",
        ),
    ];
    for (width, expected) in cases {
        let line = wrapped(width);
        assert_eq!(line, expected, "at {width} columns");
        // The count leads, so a reader sees how many rows the table holds
        // before the filter that chose them, whatever the width.
        let first = line.lines().next().unwrap_or_default();
        assert!(first.starts_with("2 of 1013 tasks"), "at {width}: {line}");
        assert_eq!(
            line.matches("2 of 1013 tasks").count(),
            1,
            "at {width}: {line}"
        );
    }
}

#[test]
fn footer_shows_the_description_of_the_view_that_supplied_the_filter() {
    assert_eq!(
        resolved_footer(&agent_config(""), Some("@agent"), 3, 1013),
        "3 of 1013 tasks · agent view (Open tasks an agent can take)"
    );
}

#[test]
fn empty_hint_widens_a_filter_that_matched_nothing() {
    assert_eq!(
        empty_hint(0, 941).as_deref(),
        Some(
            "hint: no task matches; run mt ls '' for every task, \
             or mt config show --resolved for a saved view"
        )
    );
}

#[test]
fn empty_hint_says_nothing_when_a_row_showed_or_the_project_is_empty() {
    assert_eq!(empty_hint(1, 941), None);
    assert_eq!(empty_hint(0, 0), None);
}

#[test]
fn note_supersession_spares_a_rule_that_still_owns_a_facet() {
    let before = [
        "release".to_owned(),
        "release".to_owned(),
        MATCH_ALL.to_owned(),
    ];
    let mut applied = Applied {
        rule: Some("release".to_owned()),
        filter_from: "manual".to_owned(),
        sort_from: "release".to_owned(),
        columns_from: MATCH_ALL.to_owned(),
        ..Applied::default()
    };
    applied.note_supersession(&before);
    assert!(!applied.rule_superseded);
}

#[test]
fn note_supersession_spares_a_rule_that_contributed_nothing() {
    // An empty rule was never superseded — nothing of its was taken away, so
    // saying so would name a loss that did not happen.
    let before = [
        MATCH_ALL.to_owned(),
        MATCH_ALL.to_owned(),
        MATCH_ALL.to_owned(),
    ];
    let mut applied = Applied {
        rule: Some("empty".to_owned()),
        filter_from: "manual".to_owned(),
        sort_from: MATCH_ALL.to_owned(),
        columns_from: MATCH_ALL.to_owned(),
        ..Applied::default()
    };
    applied.note_supersession(&before);
    assert!(!applied.rule_superseded);
}

#[test]
fn note_supersession_ignores_a_listing_no_rule_matched() {
    let before = [
        "default".to_owned(),
        MATCH_ALL.to_owned(),
        MATCH_ALL.to_owned(),
    ];
    let mut applied = Applied {
        rule: None,
        filter_from: "manual".to_owned(),
        ..Applied::default()
    };
    applied.note_supersession(&before);
    assert!(!applied.rule_superseded);
}

#[test]
fn bare_at_sign_is_manual_filter_not_view_ref() {
    // Kills mutant at && → || in bare_view_ref.
    // "@" has empty name after @, so bare_view_ref returns None in real code.
    // Mutant makes it return Some(""), which then errors looking up view "".
    let (_dir, project) = project_with_config(mindtape::Config::default());
    let (bundle, applied) = resolve(&project, &args(Some("@"))).unwrap();
    assert_eq!(bundle.filter, "@");
    assert_eq!(applied.filter_from, "manual");
}

#[test]
fn view_ref_with_underscore_is_valid() {
    // Kills mutant at second || → && in bare_view_ref.
    // "_" is a valid view-name character. Mutant: c.is_ascii_alphanumeric() || (c=='_' && c=='-')
    // → only alphanumeric passes, so "@foo_bar" is misrecognised as manual.
    let (_dir, project) = project(
        r#"
[views.foo_bar]
filter = "priority > 5"
"#,
    );
    let (bundle, _applied) = resolve(&project, &args(Some("@foo_bar"))).unwrap();
    assert_eq!(bundle.filter, "priority > 5");
}

#[test]
fn view_ref_with_hyphen_is_valid() {
    // Kills mutant at == → != in bare_view_ref.
    // "-" is a valid view-name char under ==; under != it is rejected.
    let (_dir, project) = project(
        r#"
[views.my-view]
filter = "priority > 5"
"#,
    );
    let (bundle, _applied) = resolve(&project, &args(Some("@my-view"))).unwrap();
    assert_eq!(bundle.filter, "priority > 5");
}
