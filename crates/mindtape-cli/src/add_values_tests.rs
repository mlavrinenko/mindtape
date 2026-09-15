//! Tests for `add` value sources: `--set`/`-f` merging, the JSON/YAML `-f`
//! dispatch (extension-driven and sniffed), env-var layering, and unknown-key
//! rejection. Template resolution and describe/slug output live in
//! `add_values_tests_templates.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use mindtape::{RawValue, discover_slots};

use super::{
    BUILTIN_TASK, collect_values, env_var_name, layer_env, parse_sniffed_source, parse_source,
    reject_unknown_keys,
};

/// Write `text` to `dir`/`name` and return the JSON/YAML `-f` value:
/// `collect_values` reads a path from disk, so extension-driven dispatch
/// needs a real file, not an in-memory string.
pub(super) fn values_file(dir: &std::path::Path, name: &str, text: &str) -> String {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn env_var_name_uppercases_and_underscores() {
    assert_eq!(env_var_name("title"), "MINDTAPE_ADD_TITLE");
    assert_eq!(env_var_name("depends-on"), "MINDTAPE_ADD_DEPENDS_ON");
}

#[test]
fn layer_env_fills_unset_skips_seeded_and_blank() {
    let specs = discover_slots(BUILTIN_TASK); // the barebones template: one `title` slot
    let lookup = |key: &str| (key == "MINDTAPE_ADD_TITLE").then(|| "From Env".to_owned());

    let mut values = BTreeMap::new();
    layer_env(&mut values, &specs, lookup);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From Env".to_owned()))
    );

    // A value already set by --set/-f is never overwritten by env.
    let mut seeded = BTreeMap::new();
    seeded.insert("title".to_owned(), RawValue::Str("Explicit".to_owned()));
    layer_env(&mut seeded, &specs, lookup);
    assert_eq!(
        seeded.get("title"),
        Some(&RawValue::Str("Explicit".to_owned()))
    );

    // A blank env value sets nothing.
    let mut blank = BTreeMap::new();
    layer_env(&mut blank, &specs, |_| Some("   ".to_owned()));
    assert!(blank.is_empty());
}

#[test]
fn layer_env_reads_declared_env_name() {
    let specs =
        discover_slots(r#"#f(t: slot("title", env: "MINDTAPE_PRINCIPAL", default: "untitled"))"#);
    let mut values = BTreeMap::new();
    layer_env(&mut values, &specs, |key| {
        (key == "MINDTAPE_PRINCIPAL").then(|| "From Declared Env".to_owned())
    });
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From Declared Env".to_owned()))
    );
}

#[test]
fn layer_env_declared_env_unset_falls_through() {
    let specs =
        discover_slots(r#"#f(t: slot("title", env: "MINDTAPE_PRINCIPAL", default: "untitled"))"#);
    // The declared variable is unset: nothing fills, and the implicit
    // `MINDTAPE_ADD_TITLE` is not consulted — a declared `env` replaces it.
    let mut values = BTreeMap::new();
    layer_env(&mut values, &specs, |key| {
        (key == "MINDTAPE_ADD_TITLE").then(|| "Implicit".to_owned())
    });
    assert!(values.is_empty());
}

#[test]
fn collect_set_overrides_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v.json");
    std::fs::write(&path, r#"{"title":"From JSON","impact":8,"flag":true}"#).unwrap();
    let values = collect_values(
        &[path.to_string_lossy().into_owned()],
        &["title=Override".to_owned()],
        &[],
    )
    .unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("Override".to_owned()))
    );
    assert_eq!(values.get("impact"), Some(&RawValue::Int(8)));
    assert_eq!(values.get("flag"), Some(&RawValue::Bool(true)));
}

#[test]
fn collect_append_operator_accumulates_into_list() {
    let values = collect_values(
        &[],
        &[
            "tags+=a".to_owned(),
            "tags+=b".to_owned(),
            "tags+=b,c".to_owned(),
        ],
        &[],
    )
    .unwrap();
    assert_eq!(
        values.get("tags"),
        Some(&RawValue::List(vec![
            "a".to_owned(),
            "b".to_owned(),
            "b,c".to_owned(), // a literal comma in a value is never split
        ]))
    );
}

#[test]
fn collect_append_operator_appends_to_json_list() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v.json");
    std::fs::write(&path, r#"{"tags":["a"]}"#).unwrap();
    let values = collect_values(
        &[path.to_string_lossy().into_owned()],
        &["tags+=b".to_owned()],
        &[],
    )
    .unwrap();
    assert_eq!(
        values.get("tags"),
        Some(&RawValue::List(vec!["a".to_owned(), "b".to_owned()]))
    );
}

#[test]
fn collect_reads_json_array_as_list() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v.json");
    std::fs::write(&path, r#"{"tags":["a","b"]}"#).unwrap();
    let values = collect_values(&[path.to_string_lossy().into_owned()], &[], &[]).unwrap();
    assert_eq!(
        values.get("tags"),
        Some(&RawValue::List(vec!["a".to_owned(), "b".to_owned()]))
    );
}

#[test]
fn collect_rejects_non_object_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v.json");
    std::fs::write(&path, "[1,2,3]").unwrap();
    assert!(collect_values(&[path.to_string_lossy().into_owned()], &[], &[]).is_err());
}

// ── YAML `-f` dispatch: extension, sniff, malformed input, tabs, blanks ──

#[test]
fn dot_yaml_extension_parses_as_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.yaml", "title: From YAML\nimpact: 8\n");
    let values = collect_values(&[path], &[], &[]).unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From YAML".to_owned()))
    );
    assert_eq!(values.get("impact"), Some(&RawValue::Int(8)));
}

#[test]
fn dot_yml_extension_parses_as_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.yml", "title: From Yml\n");
    let values = collect_values(&[path], &[], &[]).unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From Yml".to_owned()))
    );
}

#[test]
fn sniff_picks_yaml_for_non_brace_content() {
    // No filename extension to trust (the stdin `-` shape): a document with
    // no leading `{`/`[` sniffs as YAML.
    let values = parse_sniffed_source("-", "title: Sniffed\ntags:\n  - a\n  - b\n").unwrap();
    assert_eq!(
        values.get("title"),
        Some(&Some(RawValue::Str("Sniffed".to_owned())))
    );
    assert_eq!(
        values.get("tags"),
        Some(&Some(RawValue::List(vec!["a".to_owned(), "b".to_owned()])))
    );
}

#[test]
fn sniff_picks_json_for_brace_content() {
    let values = parse_sniffed_source("-", r#"{"title":"Sniffed JSON"}"#).unwrap();
    assert_eq!(
        values.get("title"),
        Some(&Some(RawValue::Str("Sniffed JSON".to_owned())))
    );
}

#[test]
fn unknown_extension_sniffs_like_stdin() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.values", "title: Unknown Extension\n");
    let values = collect_values(&[path], &[], &[]).unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("Unknown Extension".to_owned()))
    );
}

#[test]
fn malformed_yaml_extension_reports_a_useful_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.yaml", "title: [unterminated\n");
    let err = collect_values(std::slice::from_ref(&path), &[], &[])
        .unwrap_err()
        .to_string();
    assert!(err.contains(path.as_str()), "{err}");
}

#[test]
fn malformed_sniffed_input_reports_the_json_error() {
    // Leading `{` sniffs as JSON; when both the JSON and the YAML retry fail,
    // the JSON parse error is the one surfaced (see `parse_sniffed_source`).
    let err = parse_sniffed_source("-", "{not valid anything")
        .unwrap_err()
        .to_string();
    assert!(err.contains("parse JSON from -"), "{err}");
}

#[test]
fn tab_indented_json_still_parses() {
    // libyaml rejects tab indentation; a `.json`-dispatched source must keep
    // using serde_json, which accepts it.
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.json", "{\n\t\"title\": \"Tabbed\"\n}\n");
    let values = collect_values(&[path], &[], &[]).unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("Tabbed".to_owned()))
    );
}

#[test]
fn tab_indented_json_still_parses_when_sniffed() {
    let values = parse_sniffed_source("-", "{\n\t\"title\": \"Tabbed\"\n}\n").unwrap();
    assert_eq!(
        values.get("title"),
        Some(&Some(RawValue::Str("Tabbed".to_owned())))
    );
}

#[test]
fn empty_string_survives_a_yaml_values_file() {
    // Unlike the `--edit` scaffold (blank means "use the default"), a `-f`
    // file's `""` is a real value that must reach the fill map unchanged.
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.yaml", "note: \"\"\n");
    let values = collect_values(&[path], &[], &[]).unwrap();
    assert_eq!(values.get("note"), Some(&RawValue::Str(String::new())));
}

#[test]
fn empty_string_survives_a_json_values_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.json", r#"{"note":""}"#);
    let values = collect_values(&[path], &[], &[]).unwrap();
    assert_eq!(values.get("note"), Some(&RawValue::Str(String::new())));
}

#[test]
fn yaml_non_mapping_root_is_rejected_with_json_shaped_wording() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.yaml", "- a\n- b\n");
    let err = collect_values(std::slice::from_ref(&path), &[], &[])
        .unwrap_err()
        .to_string();
    assert!(
        err.contains(&format!("values from {path} must be a mapping")),
        "{err}"
    );
}

#[test]
fn yaml_multi_document_stream_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = values_file(dir.path(), "v.yaml", "---\na: 1\n---\nb: 2\n");
    assert!(collect_values(&[path], &[], &[]).is_err());
}

#[test]
fn yaml_extension_dispatch_matches_parse_source() {
    // `parse_source` is the single dispatch point `collect_values` uses per
    // source; pin its extension routing directly.
    let values = parse_source("v.yaml", "x: 1\n").unwrap();
    assert_eq!(values.get("x"), Some(&Some(RawValue::Int(1))));
}

#[test]
fn later_yaml_null_unsets_an_earlier_set_key() {
    let dir = tempfile::tempdir().unwrap();
    let first = values_file(dir.path(), "a.json", r#"{"title":"First","impact":8}"#);
    let second = values_file(dir.path(), "b.yaml", "impact: ~\n");
    let values = collect_values(&[first, second], &[], &[]).unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("First".to_owned()))
    );
    assert!(
        !values.contains_key("impact"),
        "a later explicit null must unset an earlier value: {values:?}"
    );
}

#[test]
fn reject_unknown_keys_allows_known_slots() {
    let specs = discover_slots(BUILTIN_TASK); // one `title` slot
    let mut values = BTreeMap::new();
    values.insert("title".to_owned(), RawValue::Str("Fine".to_owned()));
    assert!(reject_unknown_keys(&values, &specs).is_ok());
}

#[test]
fn reject_unknown_keys_errors_naming_the_key_and_known_slots() {
    let specs = discover_slots(BUILTIN_TASK); // one `title` slot
    let mut values = BTreeMap::new();
    values.insert("title".to_owned(), RawValue::Str("Fine".to_owned()));
    values.insert("priority".to_owned(), RawValue::Str("3".to_owned()));
    let err = reject_unknown_keys(&values, &specs)
        .unwrap_err()
        .to_string();
    assert!(err.contains("priority"), "{err}");
    assert!(err.contains("title"), "{err}");
}

// --- suggest wiring for an unknown `--set`/`-f` key
// (`truth/tasks/adopt-nearest-match-suggestions-at-the-remaining-unknown-name-errors.typ`) --

#[test]
fn reject_unknown_keys_near_miss_offers_a_suggestion() {
    let specs = discover_slots(BUILTIN_TASK); // one `title` slot
    let mut values = BTreeMap::new();
    values.insert("titel".to_owned(), RawValue::Str("Typo".to_owned()));
    let err = reject_unknown_keys(&values, &specs)
        .unwrap_err()
        .to_string();
    assert!(err.contains("did you mean: title?"), "{err}");
}

#[test]
fn reject_unknown_keys_far_miss_omits_the_suggestion() {
    let specs = discover_slots(BUILTIN_TASK); // one `title` slot
    let mut values = BTreeMap::new();
    values.insert("zzzzzzzz".to_owned(), RawValue::Str("Nope".to_owned()));
    let err = reject_unknown_keys(&values, &specs)
        .unwrap_err()
        .to_string();
    assert!(!err.contains("did you mean"), "{err}");
}

#[test]
fn reject_unknown_keys_with_no_slots_omits_the_suggestion() {
    let specs: Vec<mindtape::SlotSpec> = Vec::new();
    let mut values = BTreeMap::new();
    values.insert("anything".to_owned(), RawValue::Str("x".to_owned()));
    let err = reject_unknown_keys(&values, &specs)
        .unwrap_err()
        .to_string();
    assert!(!err.contains("did you mean"), "{err}");
    assert!(err.contains("known slots: "), "{err}");
}

/// The `--` tokens and `--set` reach the same parser, so both operators have
/// to behave identically whichever carrier delivered them. Asserting only the
/// `--set` path would let the shorthand drift into a second grammar.
#[test]
fn trailing_tokens_parse_as_set_entries_do() {
    let values = collect_values(
        &[],
        &[],
        &["title=From tokens".to_owned(), "tags+=docs".to_owned()],
    )
    .unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From tokens".to_owned()))
    );
    assert_eq!(
        values.get("tags"),
        Some(&RawValue::List(vec!["docs".to_owned()]))
    );
}

#[test]
fn trailing_tokens_apply_after_set() {
    let values = collect_values(
        &[],
        &["title=From set".to_owned()],
        &["title=From tokens".to_owned()],
    )
    .unwrap();
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From tokens".to_owned()))
    );
}

/// One list built from both carriers at once: `+=` appends rather than
/// replacing, so the carrier a token arrived through cannot reset the list.
#[test]
fn append_operator_accumulates_across_both_carriers() {
    let values = collect_values(&[], &["tags+=a".to_owned()], &["tags+=b".to_owned()]).unwrap();
    assert_eq!(
        values.get("tags"),
        Some(&RawValue::List(vec!["a".to_owned(), "b".to_owned()]))
    );
}

/// The pre-release `key[]=value` append spelling carries no diagnostic of its
/// own any more: `collect_values` accepts the bracketed text as an ordinary
/// key, and it fails downstream exactly as any other unknown key does — an
/// `unknown --set/-f key(s)` error naming it, with no pointer back to `+=`.
#[test]
fn bracket_append_spelling_is_an_unknown_key_like_any_other() {
    let values = collect_values(&[], &["tags[]=docs".to_owned()], &[]).unwrap();
    assert_eq!(
        values.get("tags[]"),
        Some(&RawValue::Str("docs".to_owned()))
    );
    let specs = discover_slots(BUILTIN_TASK); // one `title` slot, no `tags[]`
    let err = reject_unknown_keys(&values, &specs)
        .unwrap_err()
        .to_string();
    assert!(err.contains("tags[]"), "{err}");
}

#[test]
fn token_without_an_equals_errors_naming_the_token() {
    let err = collect_values(&[], &[], &["tags".to_owned()])
        .unwrap_err()
        .to_string();
    assert!(err.contains("tags"), "{err}");
    assert!(err.contains("key+=value"), "{err}");
}

/// A value may hold `=`; only the FIRST one separates. Splitting on the last
/// would corrupt any value carrying one.
#[test]
fn only_the_first_equals_separates() {
    let values = collect_values(&[], &[], &["summary=a=b=c".to_owned()]).unwrap();
    assert_eq!(
        values.get("summary"),
        Some(&RawValue::Str("a=b=c".to_owned()))
    );
}
