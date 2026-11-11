//! Tests for ordered operations: slot resolution and unknown-key rejection.
//! The source parsers and their dispatch live in `values_tests.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::{FieldOp, RawValue, discover_slots};

use super::{BUILTIN_TASK, collect_values, reject_unknown_keys};

fn ops(pairs: &[(&str, &str)]) -> Vec<(String, FieldOp)> {
    pairs
        .iter()
        .map(|(name, value)| {
            (
                (*name).to_owned(),
                FieldOp::Set(RawValue::Str((*value).to_owned())),
            )
        })
        .collect()
}

#[test]
fn reject_unknown_keys_allows_known_slots_and_writable_fields() {
    let specs = discover_slots(BUILTIN_TASK); // one `title` slot
    assert!(reject_unknown_keys(&ops(&[("title", "Fine"), ("due", "2026-01-01")]), &specs).is_ok());
}

#[test]
fn reject_unknown_keys_errors_naming_the_key_and_known_slots() {
    let specs = discover_slots(BUILTIN_TASK); // one `title` slot
    let err = crate::diag::plain(
        &reject_unknown_keys(&ops(&[("title", "Fine"), ("weights", "3")]), &specs).unwrap_err(),
    );
    assert_eq!(
        err,
        "error: unknown slot or field: 'weights'\nhint: run mt add --describe to see the slots\n"
    );
}

#[test]
fn dir_is_refused_with_the_add_or_rename_hint() {
    let specs = discover_slots(BUILTIN_TASK);
    let err =
        crate::diag::plain(&reject_unknown_keys(&ops(&[("dir", "fixes")]), &specs).unwrap_err());
    assert!(err.contains("--dir"), "{err}");
    assert!(err.contains("mt mv"), "{err}");
}

#[test]
fn an_append_to_a_scalar_writable_field_is_refused_by_name() {
    let specs = discover_slots(BUILTIN_TASK);
    let err = crate::diag::plain(
        &reject_unknown_keys(
            &[("priority".to_owned(), FieldOp::Append("8".to_owned()))],
            &specs,
        )
        .unwrap_err(),
    );
    assert!(err.contains("priority"), "{err}");
    assert!(err.contains("list"), "{err}");
}

// --- suggest wiring for an unknown `--set`/`-f` key
// (`truth/tasks/adopt-nearest-match-suggestions-at-the-remaining-unknown-name-errors.typ`) --

#[test]
fn reject_unknown_keys_near_miss_offers_a_suggestion() {
    let specs = discover_slots(BUILTIN_TASK); // one `title` slot
    let err =
        crate::diag::plain(&reject_unknown_keys(&ops(&[("titel", "Typo")]), &specs).unwrap_err());
    assert!(err.contains("hint: did you mean title?\n"), "{err}");
}

#[test]
fn reject_unknown_keys_far_miss_omits_the_suggestion() {
    let specs = discover_slots(BUILTIN_TASK); // one `title` slot
    let err = crate::diag::plain(
        &reject_unknown_keys(&ops(&[("zzzzzzzz", "Nope")]), &specs).unwrap_err(),
    );
    assert!(!err.contains("did you mean"), "{err}");
}

#[test]
fn reject_unknown_keys_with_no_slots_omits_the_suggestion() {
    let specs: Vec<mindtape::SlotSpec> = Vec::new();
    let err =
        crate::diag::plain(&reject_unknown_keys(&ops(&[("anything", "x")]), &specs).unwrap_err());
    assert!(!err.contains("did you mean"), "{err}");
    assert!(err.contains("hint: run mt add --describe"), "{err}");
}

/// The pre-release `key[]=value` append spelling carries no diagnostic of its
/// own any more: `collect_values` accepts the bracketed text as an ordinary
/// key, and it fails downstream exactly as any other unknown key does.
#[test]
fn bracket_append_spelling_is_an_unknown_key_like_any_other() {
    let collected = collect_values(&[], &["tags[]=docs".to_owned()], &[]).unwrap();
    let specs = discover_slots(BUILTIN_TASK); // one `title` slot, no `tags[]`
    let err = reject_unknown_keys(&collected.ops, &specs)
        .unwrap_err()
        .to_string();
    assert!(err.contains("tags[]"), "{err}");
}

/// Several unknown keys come out as one error: the first is the value, the
/// rest are notes, so none is lost to the first re-run.
#[test]
fn reject_unknown_keys_names_every_unknown_key() {
    let specs = discover_slots(BUILTIN_TASK);
    let err = crate::diag::plain(
        &reject_unknown_keys(&ops(&[("alpha", "1"), ("beta", "2")]), &specs).unwrap_err(),
    );
    assert!(
        err.starts_with("error: unknown slot or field: 'alpha'\n"),
        "{err}"
    );
    assert!(err.contains("note: also unknown: 'beta'\n"), "{err}");
}

const TAGS_DEFAULT: &str = r#"#f(tags: slot("tags", kind: "list", default: ("base",)))"#;

/// `+=` on a list slot starts from the template's default, not from an empty
/// list, and a later `=` drops the earlier appends.
#[test]
fn a_list_slot_append_starts_from_its_default() {
    let collected = collect_values(&[], &["tags+=a".to_owned()], &[]).unwrap();
    let slots = collected
        .slot_values(&discover_slots(TAGS_DEFAULT))
        .unwrap();
    assert_eq!(
        slots.get("tags"),
        Some(&RawValue::List(vec!["base".to_owned(), "a".to_owned()]))
    );

    let collected = collect_values(&[], &["tags+=a".to_owned(), "tags=b".to_owned()], &[]).unwrap();
    let slots = collected
        .slot_values(&discover_slots(TAGS_DEFAULT))
        .unwrap();
    assert_eq!(
        slots.get("tags"),
        Some(&RawValue::List(vec!["b".to_owned()]))
    );
}

/// An explicit base is where a following `+=` appends, whichever carrier gave
/// it, and `=` means one item for a writable list field.
#[test]
fn an_explicit_list_base_feeds_the_following_appends() {
    let collected = collect_values(&[], &["tags=b".to_owned(), "tags+=a".to_owned()], &[]).unwrap();
    let slots = collected
        .slot_values(&discover_slots(TAGS_DEFAULT))
        .unwrap();
    assert_eq!(
        slots.get("tags"),
        Some(&RawValue::List(vec!["b".to_owned(), "a".to_owned()]))
    );
}

/// A slot with no operation is absent from the fill map, so `fill` keeps the
/// template's own default instead of replacing it.
#[test]
fn an_untouched_slot_is_absent_from_the_fill_map() {
    let collected = collect_values(&[], &[], &[]).unwrap();
    let slots = collected
        .slot_values(&discover_slots(TAGS_DEFAULT))
        .unwrap();
    assert!(slots.is_empty());
}

#[test]
fn an_append_to_a_scalar_slot_is_refused_by_name() {
    let collected = collect_values(&[], &["impact+=8".to_owned()], &[]).unwrap();
    let err = collected
        .slot_values(&discover_slots(
            r#"#f(impact: slot("impact", kind: "int"))"#,
        ))
        .unwrap_err();
    let text = crate::diag::plain(&err);
    assert!(text.contains("impact"), "{text}");
    assert!(text.contains("list"), "{text}");
}

/// The ordered operations survive collection, so a writer can tell `+=` from
/// `=` instead of seeing only a collapsed list.
#[test]
fn collect_keeps_the_operations_in_argument_order() {
    let collected = collect_values(
        &[],
        &["tags+=a".to_owned()],
        &["title=T".to_owned(), "tags=b".to_owned()],
    )
    .unwrap();
    let ops: Vec<(&str, &mindtape::FieldOp)> = collected
        .ops
        .iter()
        .map(|(name, op)| (name.as_str(), op))
        .collect();
    assert_eq!(ops.len(), 3);
    assert!(matches!(ops[0].1, mindtape::FieldOp::Append(item) if item == "a"));
    assert!(matches!(ops[1].1, mindtape::FieldOp::Set(_)));
    assert!(matches!(ops[2].1, mindtape::FieldOp::Set(_)));
}

#[test]
fn a_scalar_value_renders_every_shape() {
    use crate::cmd::add::values_ops::scalar_text;
    assert_eq!(scalar_text(&RawValue::Str("x".to_owned())), "x");
    assert_eq!(scalar_text(&RawValue::Int(2)), "2");
    assert_eq!(scalar_text(&RawValue::Float(2.5)), "2.5");
    assert_eq!(scalar_text(&RawValue::Bool(true)), "true");
    assert_eq!(scalar_text(&RawValue::List(vec!["x".to_owned()])), "");
    assert_eq!(scalar_text(&RawValue::Links(Vec::new())), "");
}

#[test]
fn a_list_slot_accepts_an_array_or_one_scalar() {
    let spec = discover_slots(r#"#f(tags: slot("tags", kind: "list"))"#)
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(
        crate::cmd::add::values_ops::base_value(&spec, &RawValue::List(vec!["a".to_owned()])),
        RawValue::List(vec!["a".to_owned()])
    );
    assert_eq!(
        crate::cmd::add::values_ops::base_value(&spec, &RawValue::Int(3)),
        RawValue::List(vec!["3".to_owned()])
    );
}

#[test]
fn appending_to_a_non_list_value_starts_a_list() {
    let mut value = RawValue::Str("a".to_owned());
    crate::cmd::add::values_ops::append_item(&mut value, "b");
    assert_eq!(value, RawValue::List(vec!["b".to_owned()]));
}
