//! Tests for `add` value sources: `--set`/`-f` merging, the JSON/YAML `-f`
//! dispatch (extension-driven and sniffed), slot resolution and unknown-key
//! rejection.
//! Env-var layering lives in `cmd/add/values_tests_env.rs`. Template resolution
//! and describe/slug output in `cmd/add/values_tests_templates.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::test_support::tempdir;

use mindtape::RawValue;

use super::{collect_values, parse_sniffed_source, parse_source};
use crate::cmd::add::support_tests::{
    collect_err_at, collect_map, collect_override, collect_path, json_file, json_values_file,
    links, list, unterminated_yaml_file, values_file, yaml_values_file, yml_values_file,
};

#[test]
fn collect_set_overrides_json() {
    let dir = tempdir();
    let path = json_values_file(dir.path());
    let values = collect_override(&path);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("Override".to_owned()))
    );
    assert_eq!(values.get("impact"), Some(&RawValue::Int(8)));
    assert_eq!(values.get("flag"), Some(&RawValue::Bool(true)));
}

#[test]
fn collect_append_operator_accumulates_into_list() {
    let values = collect_map(
        &[],
        &[
            "tags+=a".to_owned(),
            "tags+=b".to_owned(),
            "tags+=b,c".to_owned(),
        ],
        &[],
    );
    assert_eq!(values.get("tags"), Some(&list(&["a", "b", "b,c"])));
}

#[test]
fn collect_append_operator_appends_to_json_list() {
    let dir = tempdir();
    let path = json_file(dir.path(), r#"{"tags":["a"]}"#);
    let values = collect_map(&[path], &["tags+=b".to_owned()], &[]);
    assert_eq!(values.get("tags"), Some(&list(&["a", "b"])));
}

#[test]
fn collect_reads_json_array_as_list() {
    let dir = tempdir();
    let path = json_file(dir.path(), r#"{"tags":["a","b"]}"#);
    let values = collect_path(&path);
    assert_eq!(values.get("tags"), Some(&list(&["a", "b"])));
}

#[test]
fn collect_reads_json_array_of_link_mappings() {
    let dir = tempdir();
    let path = json_file(
        dir.path(),
        r#"{"links":[{"kind":"surfaced-by","task":"origin.typ"},{"kind":"related","task":"other.typ","note":"why"}]}"#,
    );
    let values = collect_path(&path);
    assert_eq!(values.get("links"), Some(&links()));
}

#[test]
fn collect_rejects_a_mixed_json_list() {
    let dir = tempdir();
    let path = json_file(
        dir.path(),
        r#"{"links":["a",{"kind":"related","task":"other.typ"}]}"#,
    );
    let err = collect_err_at(&path);
    assert!(err.contains("strings or link mappings"), "{err}");
}

#[test]
fn collect_rejects_a_json_link_without_a_kind() {
    let dir = tempdir();
    let path = json_file(dir.path(), r#"{"links":[{"task":"other.typ"}]}"#);
    let err = collect_err_at(&path);
    assert!(err.contains("a link needs a kind"), "{err}");
}

#[test]
fn collect_rejects_non_object_json() {
    let dir = tempdir();
    let path = json_file(dir.path(), "[1,2,3]");
    assert!(collect_values(&[path], &[], &[]).is_err());
}

// ── YAML `-f` dispatch: extension, sniff, malformed input, tabs, blanks ──

#[test]
fn dot_yaml_extension_parses_as_yaml() {
    let dir = tempdir();
    let path = yaml_values_file(dir.path());
    let values = collect_path(&path);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From YAML".to_owned()))
    );
    assert_eq!(values.get("impact"), Some(&RawValue::Int(8)));
}

#[test]
fn dot_yml_extension_parses_as_yaml() {
    let dir = tempdir();
    let path = yml_values_file(dir.path());
    let values = collect_path(&path);
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
    assert_eq!(values.get("tags"), Some(&Some(list(&["a", "b"]))));
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
    let dir = tempdir();
    let path = values_file(dir.path(), "v.values", "title: Unknown Extension\n");
    let values = collect_path(&path);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("Unknown Extension".to_owned()))
    );
}

#[test]
fn malformed_yaml_extension_reports_a_useful_error() {
    let dir = tempdir();
    let path = unterminated_yaml_file(dir.path());
    let err = collect_err_at(&path);
    assert!(err.contains(path.as_str()), "{err}");
}

#[test]
fn malformed_sniffed_input_reports_the_json_error() {
    // Leading `{` sniffs as JSON; when both the JSON and the YAML retry fail,
    // the JSON parse error is the one surfaced (see `parse_sniffed_source`).
    let err = parse_sniffed_source("-", "{not valid anything")
        .unwrap_err()
        .to_string();
    assert!(err.starts_with("can't parse JSON: '-': "), "{err}");
}

#[test]
fn tab_indented_json_still_parses() {
    // libyaml rejects tab indentation; a `.json`-dispatched source must keep
    // using serde_json, which accepts it.
    let dir = tempdir();
    let path = values_file(dir.path(), "v.json", "{\n\t\"title\": \"Tabbed\"\n}\n");
    let values = collect_path(&path);
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
    let dir = tempdir();
    let path = values_file(dir.path(), "v.yaml", "note: \"\"\n");
    let values = collect_path(&path);
    assert_eq!(values.get("note"), Some(&RawValue::Str(String::new())));
}

#[test]
fn empty_string_survives_a_json_values_file() {
    let dir = tempdir();
    let path = values_file(dir.path(), "v.json", r#"{"note":""}"#);
    let values = collect_path(&path);
    assert_eq!(values.get("note"), Some(&RawValue::Str(String::new())));
}

#[test]
fn yaml_non_mapping_root_is_rejected_with_json_shaped_wording() {
    let dir = tempdir();
    let path = values_file(dir.path(), "v.yaml", "- a\n- b\n");
    let err = collect_err_at(&path);
    assert_eq!(err, format!("values must be a YAML mapping: '{path}'"));
}

#[test]
fn yaml_multi_document_stream_is_rejected() {
    let dir = tempdir();
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

/// The `.json` extension picks the JSON parser outright. A YAML-shaped body
/// then fails rather than silently sniffing into YAML. Pins both the `source ==
/// "-"` short-circuit and the `Some("json")` arm against removal.
#[test]
fn json_extension_forces_the_json_parser() {
    let err = parse_source("v.json", "title: not json\n").unwrap_err();
    assert!(
        err.to_string().starts_with("can't parse JSON: 'v.json': "),
        "{err}"
    );
}

/// Symmetrically, `.yaml`/`.yml` picks the YAML parser. The reported error then
/// names YAML even for brace-leading bytes that sniffing would try as JSON
/// first. Pins the `Some("yaml" | "yml")` arm against removal.
#[test]
fn yaml_extension_forces_the_yaml_parser() {
    let err = parse_source("v.yaml", "{oops\n").unwrap_err();
    assert!(
        err.to_string().starts_with("can't parse YAML: 'v.yaml': "),
        "{err}"
    );
}

/// A non-brace body goes straight to YAML. A malformed one then reports the
/// YAML error rather than going to JSON first. Pins `looks_like_json`'s false
/// branch: a constant `true` reroutes the failure through the JSON parser and
/// rewrites the message.
#[test]
fn sniffed_non_brace_malformed_input_reports_the_yaml_error() {
    let err = parse_sniffed_source("-", "title: [unterminated\n")
        .unwrap_err()
        .to_string();
    assert!(err.starts_with("can't parse YAML: '-': "), "{err}");
}

#[test]
fn later_yaml_null_unsets_an_earlier_set_key() {
    let dir = tempdir();
    let first = values_file(dir.path(), "a.json", r#"{"title":"First","impact":8}"#);
    let second = values_file(dir.path(), "b.yaml", "impact: ~\n");
    let values = collect_map(&[first, second], &[], &[]);
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
fn later_values_source_wins_over_an_earlier_one() {
    let dir = tempdir();
    let first = values_file(dir.path(), "a.yaml", "title: First\n");
    let second = values_file(dir.path(), "b.yaml", "title: Second\n");
    let values = collect_map(&[first, second], &[], &[]);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("Second".to_owned()))
    );
}
/// The `--` tokens and `--set` reach the same parser, so both operators have
/// to behave identically whichever carrier delivered them. Asserting only the
/// `--set` path would let the shorthand drift into a second grammar.
#[test]
fn trailing_tokens_parse_as_set_entries_do() {
    let values = collect_map(
        &[],
        &[],
        &["title=From tokens".to_owned(), "tags+=docs".to_owned()],
    );
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From tokens".to_owned()))
    );
    assert_eq!(values.get("tags"), Some(&list(&["docs"])));
}

#[test]
fn trailing_tokens_apply_after_set() {
    let values = collect_map(
        &[],
        &["title=From set".to_owned()],
        &["title=From tokens".to_owned()],
    );
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From tokens".to_owned()))
    );
}

/// One list built from both carriers at once: `+=` appends rather than
/// replacing, so the carrier a token arrived through cannot reset the list.
#[test]
fn append_operator_accumulates_across_both_carriers() {
    let values = collect_map(&[], &["tags+=a".to_owned()], &["tags+=b".to_owned()]);
    assert_eq!(values.get("tags"), Some(&list(&["a", "b"])));
}

/// The pre-release `key[]=value` append spelling carries no diagnostic of its
/// own any more. `collect_values` accepts the bracketed text as an ordinary
/// key, and it fails downstream exactly as any other unknown key does. The
/// error is `unknown slot or field`, it cites the key, and it carries no
/// pointer back to `+=`.
#[test]
fn token_without_an_equals_errors_naming_the_token() {
    let err = collect_values(&[], &[], &["tags".to_owned()])
        .unwrap_err()
        .to_string();
    assert!(err.contains("tags"), "{err}");
    assert!(err.contains("key+=value"), "{err}");
}

/// Every unknown key comes out as one error: the first is the value, the rest
/// are notes, so none disappears on the first re-run.
#[test]
fn only_the_first_equals_separates() {
    let values = collect_map(&[], &[], &["summary=a=b=c".to_owned()]);
    assert_eq!(
        values.get("summary"),
        Some(&RawValue::Str("a=b=c".to_owned()))
    );
}
