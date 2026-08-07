//! `contains()`'s unified needle-first grammar: the default haystack
//! (`title`, `id`, `body`), explicit fields (one or several), the body
//! haystack's marker-free guarantee, and the two retired spellings
//! (`contains(field, "substr")`, `search("query")`) as hard errors. Lives in
//! its own module so `filter_tests.rs` stays under the linecop line cap.
#![allow(clippy::unwrap_used)]

use super::{hit, hit_views, sample};
use crate::config::Config;
use crate::filter::{Filter, FilterError};
use crate::model::Task;
use crate::parse_config;
use crate::test_support::noted_edge;

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

// --- `[ls] contains-fields`: the configurable default haystack (see
// `filter_predicates::default_contains_fields`) ---

fn config_with_contains_fields(fields: &str) -> Config {
    parse_config(&format!("[ls]\ncontains-fields = [{fields}]\n")).unwrap()
}

#[test]
fn default_haystack_unchanged_when_contains_fields_unset() {
    // No `[ls] contains-fields` at all — same shipped `title`/`id`/`body`
    // sweep as `Config::default()`.
    let cfg = parse_config("").unwrap();
    let mut task = sample();
    task.title = "Nothing".to_owned();
    task.id = "t1".to_owned();
    task.body = "the tombstone note".to_owned();
    assert!(hit_views("contains(\"tombstone\")", &task, &cfg));
}

#[test]
fn narrowed_default_excludes_id_and_body() {
    // A project narrowing `contains-fields` to `title` only.
    let cfg = config_with_contains_fields("\"title\"");
    let mut task = sample();
    task.title = "unrelated".to_owned();
    task.id = "matchable-id".to_owned();
    task.body = "matchable body text".to_owned();
    assert!(!hit_views("contains(\"matchable\")", &task, &cfg));
    task.title = "matchable title".to_owned();
    assert!(hit_views("contains(\"matchable\")", &task, &cfg));
}

#[test]
fn widened_default_includes_note_and_tags() {
    // A project widening `contains-fields` to add `note` and `tags`.
    let cfg = config_with_contains_fields("\"title\", \"id\", \"body\", \"note\", \"tags\"");
    let mut task = sample();
    task.title = "unrelated".to_owned();
    task.id = "t1".to_owned();
    task.body = "nothing".to_owned();
    task.links = vec![noted_edge("other.typ", "related", "hunt the bug here")];
    assert!(hit_views("contains(\"bug\")", &task, &cfg));
    task.links.clear();
    task.tags = vec!["needs-bugfix".to_owned()];
    assert!(hit_views("contains(\"bugfix\")", &task, &cfg));
}

#[test]
fn explicit_field_overrides_a_narrowed_default() {
    // `contains("x", note)` searches notes even though the configured default
    // omits them.
    let cfg = config_with_contains_fields("\"title\"");
    let mut task = sample();
    task.title = "unrelated".to_owned();
    task.links = vec![noted_edge("other.typ", "related", "bug lurking here")];
    assert!(!hit_views("contains(\"bug\")", &task, &cfg));
    assert!(hit_views("contains(\"bug\", note)", &task, &cfg));
}

#[test]
fn note_field_matches_a_link_note_invisible_to_the_shipped_default() {
    let mut task = sample();
    task.links = vec![noted_edge("other.typ", "related", "hunt the bug here")];
    // The shipped default (title/id/body) never sees a link note.
    assert!(!hit("contains(\"bug\")", &task));
    assert!(hit("contains(\"bug\", note)", &task));
}

#[test]
fn note_field_joins_every_link_note() {
    let mut task = sample();
    task.links = vec![
        noted_edge("a.typ", "related", "first note"),
        noted_edge("b.typ", "depends-on", "second note mentions bug"),
    ];
    assert!(hit("contains(\"bug\", note)", &task));
}

#[test]
fn tags_field_matches_authored_tags() {
    let mut task = sample();
    task.tags = vec!["needs-bugfix".to_owned()];
    task.derived_tags = Vec::new();
    assert!(hit("contains(\"bugfix\", tags)", &task));
}

#[test]
fn tags_field_matches_derived_tags() {
    let mut task = sample();
    task.tags = Vec::new();
    task.derived_tags = vec!["needs-bugfix".to_owned()];
    assert!(hit("contains(\"bugfix\", tags)", &task));
}
