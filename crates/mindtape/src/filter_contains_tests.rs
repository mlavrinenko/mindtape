//! `contains()`'s unified needle-first grammar: the default haystack
//! (`title`, `id`, `body`), explicit fields (one or several), the body
//! haystack's marker-free guarantee, and the two retired spellings
//! (`contains(field, "substr")`, `search("query")`) as hard errors. Lives in
//! its own module so `filter_tests.rs` stays under the linecop line cap.
#![allow(clippy::unwrap_used)]

use super::{hit, sample};
use crate::config::Config;
use crate::filter::{Filter, FilterError};
use crate::model::Task;

fn err(expr: &str, task: &Task) -> FilterError {
    Filter::parse(expr)
        .unwrap()
        .matches(task, &Config::default())
        .unwrap_err()
}

#[test]
fn body_only_match() {
    // The needle lives only in the prose body — neither title nor id.
    let mut task = sample();
    task.body = "left behind a tombstone entry".to_owned();
    assert!(hit("contains(\"tombstone\", body)", &task));
    assert!(hit("contains(\"tombstone\")", &task)); // default haystack covers body too
}

#[test]
fn marker_head_text_does_not_leak_into_body() {
    // A value that lives only in a head field (here: a tag, part of the
    // `#show: task.with(...)` marker in the source file) must not surface
    // through `body` — `Task.body` is already marker-free, so a hit there
    // would mean the two got conflated.
    let mut task = sample();
    task.tags = vec!["quarantine".to_owned()];
    task.body = "unrelated prose".to_owned();
    assert!(!hit("contains(\"quarantine\", body)", &task));
    assert!(!hit("contains(\"quarantine\")", &task));
}

#[test]
fn explicit_single_field() {
    let task = sample(); // title "Add Auth", body ""
    assert!(hit("contains(\"auth\", title)", &task));
    assert!(!hit("contains(\"auth\", body)", &task));
    assert!(!hit("contains(\"xyz\", title)", &task));
}

#[test]
fn explicit_several_fields() {
    let mut title_hit = sample();
    title_hit.body = "nothing here".to_owned();
    assert!(hit("contains(\"auth\", title, body)", &title_hit)); // title "Add Auth"

    let mut body_hit = sample();
    body_hit.title = "Unrelated".to_owned();
    body_hit.body = "mentions auth in prose".to_owned();
    assert!(hit("contains(\"auth\", title, body)", &body_hit));

    let mut neither = sample();
    neither.title = "Unrelated".to_owned();
    neither.body = "still nothing".to_owned();
    assert!(!hit("contains(\"auth\", title, body)", &neither));
}

#[test]
fn default_haystack_is_title_id_and_body_only() {
    let task = sample(); // id "add-auth", title "Add Auth"
    assert!(hit("contains(\"add-auth\")", &task)); // id
    assert!(hit("contains(\"Auth\")", &task)); // title

    let mut body_only = sample();
    body_only.title = "Nothing".to_owned();
    body_only.id = "t1".to_owned();
    body_only.body = "the tombstone note".to_owned();
    assert!(hit("contains(\"tombstone\")", &body_only));

    // `kind` is a FIELDS member and explicitly searchable, but not part of
    // the default sweep.
    let mut kind_only = sample();
    kind_only.title = "Nothing".to_owned();
    kind_only.id = "t1".to_owned();
    kind_only.kind = Some("umbrella-thing".to_owned());
    assert!(!hit("contains(\"umbrella\")", &kind_only));
    assert!(hit("contains(\"umbrella\", kind)", &kind_only));
}

#[test]
fn unknown_field_name_errors() {
    let got = err("contains(\"x\", bogus)", &sample());
    assert!(
        matches!(&got, FilterError::UnknownField(name) if name == "bogus"),
        "{got:?}"
    );
}

#[test]
fn unknown_field_name_errors_even_after_an_earlier_field_already_matched() {
    // Field names must be validated in full before any of them is matched —
    // an `||` fold over resolved values would short-circuit on `title`'s hit
    // and never reach `bogus`, silently accepting an invalid filter.
    let task = sample(); // title "Add Auth"
    let got = err("contains(\"auth\", title, bogus)", &task);
    assert!(
        matches!(&got, FilterError::UnknownField(name) if name == "bogus"),
        "{got:?}"
    );
}

#[test]
fn legacy_field_first_order_is_rejected() {
    // The retired `contains(field, "substr")` order: an identifier where the
    // needle belongs.
    let got = err("contains(title, \"bug\")", &sample());
    assert!(
        matches!(&got, FilterError::Invalid(msg) if msg.contains("needle first")),
        "{got:?}"
    );
}

#[test]
fn legacy_search_is_rejected() {
    let got = err("search(\"bug\")", &sample());
    assert!(
        matches!(&got, FilterError::Invalid(msg) if msg.contains("contains")),
        "{got:?}"
    );
}
