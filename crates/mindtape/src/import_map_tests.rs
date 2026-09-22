#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use serde_json::json;
use typst_harvest::HVal;

use super::*;
use crate::canonical::{
    CanonicalLink, CanonicalNum, CanonicalPriority, CanonicalStatus, CanonicalTask,
};
use crate::config::{Config, parse_config};
use crate::numeric::{InputValue, NumValue};
use crate::recur::{CanonicalRecur, CanonicalRule};

fn canonical(title: &str) -> CanonicalTask {
    CanonicalTask {
        id: "c1".to_owned(),
        title: title.to_owned(),
        ..Default::default()
    }
}

/// Legend `proposed`/`done`, a `backend`-only tag allow-list, a `kind` enum
/// of `bug`/`feature`, and a `Summary`-requiring body-header rule — the
/// shared fixture for every test that does not need a priority framework.
fn config() -> Config {
    parse_config(
        r#"
[[tasks.status]]
value = "proposed"
[[tasks.status]]
value = "done"
terminal = true

[tasks.tags]
allowed = ["backend"]

[tasks.fields.kind]
values = ["bug", "feature"]

[[tasks.body.headers.rules]]
path = ["Summary"]
action = "unique"
"#,
    )
    .unwrap()
}

fn strict_ice_config() -> Config {
    parse_config("[tasks.priority]\nmin = 0\nmax = 9\nframework = \"ice\"\nallow-manual = false\n")
        .unwrap()
}

fn ice_inputs(impact: f64, confidence: f64, ease: f64) -> BTreeMap<String, serde_json::Value> {
    BTreeMap::from([
        ("impact".to_owned(), json!(impact)),
        ("confidence".to_owned(), json!(confidence)),
        ("ease".to_owned(), json!(ease)),
    ])
}

// --- priority ----------------------------------------------------------

#[test]
fn framework_priority_is_preserved_with_converted_inputs() {
    let mut c = canonical("Ship it");
    c.priority = Some(CanonicalPriority::Framework {
        framework: "ice".to_owned(),
        inputs: ice_inputs(8.0, 0.75, 4.0),
        score: Some(2.0),
    });
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    match mapped.task.priority {
        Some(NumValue::Framework { name, inputs }) => {
            assert_eq!(name, "ice");
            assert_eq!(inputs.get("impact"), Some(&InputValue::Num(8.0)));
            assert_eq!(inputs.get("confidence"), Some(&InputValue::Num(0.75)));
            assert_eq!(inputs.get("ease"), Some(&InputValue::Num(4.0)));
        }
        other => panic!("expected a framework priority, got {other:?}"),
    }
    assert!(
        !mapped.report.has_violations(),
        "unexpected violations: {:?}",
        mapped.report.entries
    );
}

#[test]
fn unknown_priority_framework_is_reported() {
    let mut c = canonical("x");
    c.priority = Some(CanonicalPriority::Framework {
        framework: "nope".to_owned(),
        inputs: BTreeMap::new(),
        score: None,
    });
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert!(mapped.report.entries.iter().any(|e| e.field == "priority"
        && e.kind == EntryKind::Violated
        && e.message.contains("unknown")));
}

#[test]
fn priority_input_out_of_bounds_is_reported() {
    let mut c = canonical("x");
    c.priority = Some(CanonicalPriority::Framework {
        framework: "ice".to_owned(),
        inputs: ice_inputs(999.0, 0.5, 5.0),
        score: None,
    });
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "priority" && e.kind == EntryKind::Violated)
    );
}

#[test]
fn strict_framework_project_rejects_scalar_priority() {
    let mut c = canonical("x");
    c.priority = Some(CanonicalPriority::Num(5.0));
    let mapped = map_canonical(&c, &strict_ice_config(), &MapOptions::default());
    assert_eq!(mapped.task.priority, Some(NumValue::Num(5.0)));
    assert!(mapped.report.entries.iter().any(|e| e.field == "priority"
        && e.kind == EntryKind::Violated
        && e.message.contains("manual")));
}

// --- difficulty ----------------------------------------------------------

#[test]
fn difficulty_out_of_scale_is_reported_but_kept() {
    let mut c = canonical("x");
    c.difficulty = Some(CanonicalNum::Num(999.0));
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert_eq!(mapped.task.difficulty, Some(NumValue::Num(999.0)));
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "difficulty" && e.kind == EntryKind::Violated)
    );
}

#[test]
fn difficulty_in_scale_is_clean() {
    let mut c = canonical("x");
    c.difficulty = Some(CanonicalNum::Num(3.0));
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert!(!mapped.report.has_violations());
}

// --- status ----------------------------------------------------------

#[test]
fn missing_status_defaults_to_initial_legend_value() {
    let c = canonical("x");
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    let status = mapped.task.status.expect("status defaulted");
    assert_eq!(status.name, "proposed");
    assert!(status.date.is_some());
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "status" && e.kind == EntryKind::Defaulted)
    );
}

#[test]
fn status_without_date_stamps_today() {
    let mut c = canonical("x");
    c.status = Some(CanonicalStatus {
        name: "proposed".to_owned(),
        date: None,
        note: None,
    });
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    let status = mapped.task.status.expect("status present");
    assert_eq!(status.name, "proposed");
    assert!(status.date.is_some());
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "status.date" && e.kind == EntryKind::Defaulted)
    );
}

#[test]
fn out_of_legend_status_is_reported_not_silently_accepted() {
    let mut c = canonical("x");
    c.status = Some(CanonicalStatus {
        name: "bogus".to_owned(),
        date: Some("2026-01-01".to_owned()),
        note: None,
    });
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert_eq!(
        mapped.task.status.as_ref().map(|s| s.name.as_str()),
        Some("bogus")
    );
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "status" && e.kind == EntryKind::Violated)
    );
}

#[test]
fn status_map_option_renames_before_legend_check() {
    let mut c = canonical("x");
    c.status = Some(CanonicalStatus {
        name: "todo".to_owned(),
        date: Some("2026-01-01".to_owned()),
        note: None,
    });
    let mut opts = MapOptions::default();
    opts.status_map
        .insert("todo".to_owned(), "proposed".to_owned());
    let mapped = map_canonical(&c, &config(), &opts);
    assert_eq!(
        mapped.task.status.as_ref().map(|s| s.name.as_str()),
        Some("proposed")
    );
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "status" && e.kind == EntryKind::Mapped)
    );
    assert!(!mapped.report.has_violations());
}

// --- tags ----------------------------------------------------------

#[test]
fn out_of_allowlist_tag_is_reported_and_kept_by_default() {
    let mut c = canonical("x");
    c.tags = vec!["backend".to_owned(), "frontend".to_owned()];
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert_eq!(
        mapped.task.tags,
        vec!["backend".to_owned(), "frontend".to_owned()]
    );
    assert!(mapped.report.entries.iter().any(|e| e.field == "tags"
        && e.kind == EntryKind::Violated
        && e.message.contains("frontend")));
}

#[test]
fn out_of_allowlist_tag_is_dropped_with_option() {
    let mut c = canonical("x");
    c.tags = vec!["backend".to_owned(), "frontend".to_owned()];
    let opts = MapOptions {
        drop_unmapped_tags: true,
        ..MapOptions::default()
    };
    let mapped = map_canonical(&c, &config(), &opts);
    assert_eq!(mapped.task.tags, vec!["backend".to_owned()]);
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "tags" && e.kind == EntryKind::Dropped)
    );
}

// --- kind ----------------------------------------------------------

#[test]
fn kind_out_of_enum_is_reported_but_kept() {
    let mut c = canonical("x");
    c.kind = Some("chore".to_owned());
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert_eq!(mapped.task.kind.as_deref(), Some("chore"));
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "kind" && e.kind == EntryKind::Violated)
    );
}

#[test]
fn kind_in_enum_is_clean() {
    let mut c = canonical("x");
    c.kind = Some("bug".to_owned());
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert!(!mapped.report.has_violations());
}

// --- body ----------------------------------------------------------

#[test]
fn empty_body_synthesizes_summary_when_required() {
    let c = canonical("Fix the thing");
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert!(mapped.task.body.starts_with("== Summary"));
    assert!(mapped.task.body.contains("Fix the thing"));
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "body" && e.kind == EntryKind::Defaulted)
    );
}

#[test]
fn empty_body_stays_empty_without_a_summary_rule() {
    let c = canonical("x");
    let mapped = map_canonical(&c, &Config::default(), &MapOptions::default());
    assert_eq!(mapped.task.body, "");
}

#[test]
fn nonempty_body_passes_through_untouched() {
    let mut c = canonical("x");
    c.body = "== Summary\n\nAlready written.".to_owned();
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert_eq!(mapped.task.body, "== Summary\n\nAlready written.");
    assert!(!mapped.report.entries.iter().any(|e| e.field == "body"));
}

// --- extras ----------------------------------------------------------

#[test]
fn extras_carry_x_ns_keys_through() {
    let mut c = canonical("x");
    c.extras.insert(
        "x-speconaut".to_owned(),
        json!({"kind": "story", "count": 3}),
    );
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert_eq!(mapped.report.carried_extras, vec!["x-speconaut".to_owned()]);
    match mapped.task.extras.get("x-speconaut") {
        Some(HVal::Dict(dict)) => {
            assert_eq!(dict.get("kind"), Some(&HVal::Str("story".to_owned())));
            assert_eq!(dict.get("count"), Some(&HVal::Int(3)));
        }
        other => panic!("expected a dict extra, got {other:?}"),
    }
}

#[test]
fn extras_reject_keys_missing_the_x_prefix() {
    let mut c = canonical("x");
    c.extras.insert("foo".to_owned(), json!(1));
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert!(!mapped.task.extras.contains_key("foo"));
    assert!(
        mapped
            .report
            .entries
            .iter()
            .any(|e| e.field == "extras" && e.kind == EntryKind::Violated)
    );
}

#[test]
fn json_to_hval_inverts_hval_to_json_for_non_date_variants() {
    let value = HVal::Dict(BTreeMap::from([
        ("n".to_owned(), HVal::Null),
        ("b".to_owned(), HVal::Bool(true)),
        ("i".to_owned(), HVal::Int(7)),
        ("f".to_owned(), HVal::Float(1.5)),
        ("s".to_owned(), HVal::Str("hi".to_owned())),
        (
            "a".to_owned(),
            HVal::Array(vec![HVal::Int(1), HVal::Int(2)]),
        ),
    ]));
    let as_json = crate::canonical::hval_to_json(&value);
    assert_eq!(json_to_hval(&as_json), value);
}

// --- links / identity ----------------------------------------------------

#[test]
fn links_carry_through_unchanged() {
    let mut c = canonical("x");
    c.links.push(CanonicalLink {
        task: "other.typ".to_owned(),
        kind: "depends-on".to_owned(),
        note: None,
    });
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert_eq!(mapped.task.links.len(), 1);
    assert_eq!(mapped.task.links[0].task, "other.typ");
    assert_eq!(mapped.task.links[0].kind, "depends-on");
}

#[test]
fn recur_carries_through_for_an_every_period() {
    let mut c = canonical("x");
    c.recur = Some(CanonicalRecur {
        rule: CanonicalRule::Period {
            unit: "weeks".to_owned(),
            value: 2,
            on: vec!["mon".to_owned()],
        },
        anchor: Some("due".to_owned()),
        until: None,
        count: None,
        materialize: None,
    });
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert_eq!(mapped.task.recur, Some(c.recur.unwrap().to_recur()));
}

#[test]
fn recur_carries_through_for_the_rrule_escape_hatch() {
    let mut c = canonical("x");
    c.recur = Some(CanonicalRecur {
        rule: CanonicalRule::Rrule {
            raw: "FREQ=MONTHLY;BYDAY=1MO".to_owned(),
        },
        anchor: None,
        until: None,
        count: None,
        materialize: None,
    });
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert_eq!(mapped.task.recur, Some(c.recur.unwrap().to_recur()));
}

#[test]
fn absent_recur_maps_to_none() {
    let mapped = map_canonical(&canonical("x"), &config(), &MapOptions::default());
    assert_eq!(mapped.task.recur, None);
}

#[test]
fn mapped_task_carries_the_canonical_id_with_a_placeholder_path() {
    let c = canonical("x");
    let mapped = map_canonical(&c, &config(), &MapOptions::default());
    assert_eq!(mapped.task.id, "c1");
    assert_eq!(mapped.task.file, std::path::PathBuf::new());
}

// --- stem/slug ----------------------------------------------------------

#[test]
fn slug_from_title_lowercases_and_collapses() {
    assert_eq!(slug_from_title("Fix Login Bug!!"), "fix-login-bug");
}

#[test]
fn slug_from_title_drops_apostrophes_instead_of_slugging_them() {
    assert_eq!(
        slug_from_title("make a page's active argument an error"),
        "make-a-pages-active-argument-an-error"
    );
    assert_eq!(slug_from_title("Ada’s drills"), "adas-drills");
}

#[test]
fn stem_for_title_defaults_to_bare_slug() {
    let config = Config::default();
    assert_eq!(
        stem_for_title("Fix Login Bug", &config, (2026, 7, 13)).unwrap(),
        "fix-login-bug"
    );
}

#[test]
fn stem_for_title_expands_the_naming_template() {
    let config = parse_config("[tasks.naming]\nstem = \"{date}-{slug}\"\n").unwrap();
    assert_eq!(
        stem_for_title("Fix bug", &config, (2026, 7, 13)).unwrap(),
        "2026-07-13-fix-bug"
    );
}
