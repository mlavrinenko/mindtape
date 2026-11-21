//! End-to-end tests for `mt dicts`: the listing with each dictionary's count,
//! case and source, one dictionary's entries, and the two JSON shapes.
#![allow(clippy::indexing_slicing)]

use crate::support;

use support::{json, run_text, scaffold_project, stdout};

/// Two inline dictionaries, one of each case, and one that unions the two
/// built-in word lists.
const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n\
    [dicts.labels]\nvalues = [\"BILLING\", \"Auth\"]\ncase = \"exact\"\n\n\
    [dicts.verbs]\nvalues = [\"vote\", \"Rose\", \"rotate\"]\n\n\
    [dicts.all]\ninclude = [\"mindtape-verbs\", \"mindtape-dev-verbs\"]\n";

fn project() -> tempfile::TempDir {
    scaffold_project(CONFIG)
}

/// The line `mt dicts` prints for `name`.
fn line<'a>(text: &'a str, name: &str) -> &'a str {
    text.lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap_or_else(|| panic!("no line for {name:?} in:\n{text}"))
}

#[test]
fn lists_each_dictionary_with_count_case_and_source() {
    let dir = project();
    let text = stdout(dir.path(), &["dicts"]);
    for (name, count) in [("labels", "2"), ("verbs", "3"), ("all", "818")] {
        assert_eq!(
            line(&text, name).split_whitespace().nth(1),
            Some(count),
            "{text}"
        );
    }
    assert!(line(&text, "labels").contains("case exact"), "{text}");
    assert!(line(&text, "labels").ends_with("inline"), "{text}");
    assert!(line(&text, "verbs").contains("case fold"), "{text}");
    assert!(
        line(&text, "all").contains("include: mindtape-verbs, mindtape-dev-verbs"),
        "{text}"
    );
    assert!(line(&text, "mindtape-verbs").ends_with("inline"), "{text}");
}

#[test]
fn one_name_prints_its_entries() {
    let dir = project();
    for (name, entries) in [
        ("labels", "Auth\nBILLING\n"),
        ("verbs", "rose\nrotate\nvote\n"),
    ] {
        assert_eq!(stdout(dir.path(), &["dicts", name]), entries, "{name}");
    }
}

#[test]
fn json_carries_the_source_and_the_included_names() {
    let dir = project();
    let value = json(dir.path(), &["dicts", "--json"]);
    mindtape::test_support::assert_schema_at(&value, "dicts", 2);
    let dicts = &value["dicts"];
    assert_eq!(
        dicts["labels"],
        serde_json::json!({
            "case": "exact", "count": 2, "source": "inline", "include": [],
        })
    );
    assert_eq!(
        dicts["all"],
        serde_json::json!({
            "case": "fold",
            "count": 818,
            "source": "include",
            "include": ["mindtape-verbs", "mindtape-dev-verbs"],
        })
    );
}

#[test]
fn an_unknown_name_hints_at_the_nearest() {
    let dir = project();
    let (ok, text) = run_text(dir.path(), &["dicts", "verbz"]);
    assert!(!ok, "{text}");
    assert!(text.contains("error: unknown dict: 'verbz'"), "{text}");
    assert!(text.contains("hint: did you mean verbs?"), "{text}");
    assert!(text.contains("hint: use one of:"), "{text}");
}
