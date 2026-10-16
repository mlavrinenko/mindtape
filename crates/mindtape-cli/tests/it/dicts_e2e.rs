//! `mt dicts` lists a project's dictionaries and the entries of one, and a
//! rule refused because `in_dict()` missed names the dictionary and the
//! nearest entries (`truth/tasks/show-the-allowed-words-of-a-dictionary-from-the-cli.typ`).
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::disallowed_methods
)]

use crate::support;

use support::{assert_stderr, mt, scaffold_project};

/// Two dictionaries, one per case rule, and a rule that reads one. The verbs
/// sit at one edit from `Rote`, except `rotate`, so the nearest three are a
/// tie that only dictionary order breaks.
const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n\
    [dicts.verbs]\nvalues = [\"vote\", \"Rose\", \"rotate\", \"rite\", \"rate\", \"note\"]\n\n\
    [dicts.labels]\nvalues = [\"BILLING\", \"Auth\"]\ncase = \"exact\"\n\n\
    [[tasks.rules]]\nname = \"imperative-title\"\n\
    assert = 'in_dict(\"verbs\", word(title, 1))'\n\
    message = \"start the title with a verb\"\nseverity = \"error\"\n";

fn stdout(args: &[&str]) -> String {
    let dir = scaffold_project(CONFIG);
    let out = mt(dir.path()).args(args).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    String::from_utf8(out.stdout).unwrap()
}

fn json(args: &[&str]) -> serde_json::Value {
    serde_json::from_str(&stdout(args)).unwrap()
}

/// A near miss names the three nearest verbs; a far one names none.
#[test]
fn a_refused_title_names_the_dict_and_the_nearest_entries() {
    for (word, near) in [
        ("Rote", "; did you mean note, rate or rite?"),
        ("Zzyzx", ""),
    ] {
        let dir = scaffold_project(CONFIG);
        let title = format!("title={word} the keys");
        assert_stderr(
            dir.path(),
            &["add", "--name", "keys", "--set", &title],
            1,
            &format!(
                "error: <new task from template 'task'>:3: rules.imperative-title: \
                 start the title with a verb\n\
                 hint: '{word}' is not in dict verbs{near}\n\
                 hint: run mt dicts verbs to see its entries\n"
            ),
        );
    }
}

#[test]
fn check_json_carries_the_dict_hints() {
    let dir = scaffold_project(CONFIG);
    mindtape::test_support::write_file(
        dir.path(),
        "tasks/keys.typ",
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(title: \"Rote the keys\", status: wip(2026, 6, 1))\n",
    );
    let out = mt(dir.path()).args(["check", "--json"]).output().unwrap();
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let error = &report["problems"][0];
    assert_eq!(error["rule"], "rules.imperative-title", "{report}");
    assert_eq!(
        error["hints"],
        serde_json::json!([
            "'Rote' is not in dict verbs; did you mean note, rate or rite?",
            "run mt dicts verbs to see its entries",
        ])
    );
}

#[test]
fn dicts_lists_each_dict_with_its_count_and_case() {
    let out = stdout(&["dicts"]);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 4, "{out}");
    assert_eq!(lines[0], "labels              2 entries, case exact");
    assert!(lines[1].starts_with("mindtape-dev-verbs  "), "{out}");
    assert!(lines[2].starts_with("mindtape-verbs      "), "{out}");
    assert!(lines[2].ends_with(" entries, case fold"), "{out}");
    assert_eq!(lines[3], "verbs               6 entries, case fold");
}

#[test]
fn dicts_name_lists_the_compiled_entries_one_per_line() {
    assert_eq!(
        stdout(&["dicts", "verbs"]),
        "note\nrate\nrite\nrose\nrotate\nvote\n"
    );
    assert_eq!(stdout(&["dicts", "labels"]), "Auth\nBILLING\n");
}

#[test]
fn dicts_name_lists_what_include_brings_in() {
    let entries = stdout(&["dicts", "mindtape-dev-verbs"]);
    assert!(entries.lines().any(|line| line == "refactor"), "{entries}");
}

#[test]
fn dicts_json_maps_each_name_to_its_case_and_count() {
    let value = json(&["dicts", "--json"]);
    assert_eq!(
        value["labels"],
        serde_json::json!({"case": "exact", "count": 2})
    );
    assert_eq!(
        value["verbs"],
        serde_json::json!({"case": "fold", "count": 6})
    );
    assert_eq!(value["mindtape-verbs"]["case"], "fold");
}

#[test]
fn dicts_name_json_carries_the_entries() {
    assert_eq!(
        json(&["dicts", "labels", "--json"]),
        serde_json::json!({
            "name": "labels", "case": "exact", "count": 2, "entries": ["Auth", "BILLING"],
        })
    );
}

#[test]
fn an_unknown_dict_is_a_usage_error_naming_the_known_ones() {
    let dir = scaffold_project(CONFIG);
    assert_stderr(
        dir.path(),
        &["dicts", "verbz"],
        2,
        "error: unknown dict: 'verbz'\n\
         hint: did you mean verbs?\n\
         hint: use one of: labels, mindtape-dev-verbs, mindtape-verbs, verbs\n",
    );
}
