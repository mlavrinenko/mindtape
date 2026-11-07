//! End-to-end tests for `mt explain`: the bare index, the four listings, and
//! the removal of the four commands it replaces.
//!
//! `truth/tasks/fold-the-project-listings-into-mt-explain.typ` is the change
//! these pin. The rules engine itself, and the breach each rule prints, stay
//! in `rules_e2e.rs`; this file is about the reading surface.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::test_support::write_file;

use crate::support;

use support::{mt, run_text, scaffold_project};

/// A pack holding a task rule the project overrides, one it disables, a body
/// rule and a commit rule.
const PACK: &str = r#"
[[tasks.rules]]
name = "verb-title"
assert = 'has(title)'
message = "start the title with a verb"

[[tasks.rules]]
name = "no-todo"
assert = 'has(title)'
message = "drop the TODO"

[[tasks.body.headers.rules]]
name = "one-problem"
action = "unique"
path = ["Problem"]
message = "keep one Problem section"

[[commits.rules]]
name = "names-a-task"
assert = 'has(refs)'
message = "name the task in a Refs: footer"
"#;

/// The project: it extends [`PACK`], overrides `verb-title` with its own
/// message and severity, disables `no-todo`, adds an unnamed rule with no
/// message, gates `done` with a flip action that carries a message, declares
/// tags, views, a dictionary and a collector, and puts `default` last where a
/// list order might otherwise hide it.
const CONFIG: &str = r#"extends = ["./pack.toml"]
disable = ["no-todo"]

[tasks]
roots = ["tasks"]

[tasks.tags]
allowed = ["cli", "default", "mt"]

[[tasks.rules]]
name = "verb-title"
when = '!terminal'
assert = 'has(title)'
message = "open the title with an imperative verb"
severity = "error"

[[tasks.rules]]
assert = 'has(title)'

[[tasks.collectors]]
name = "changes"
returns = ["path"]
query = { section = "Changes", pattern = "`([^`]+)`" }

[[on.flip]]
when = 'into("done")'
actions = [{ gate = true, message = "run the tests, then flip again" }]

[views.blocked]
filter = "blocked"

[views.default]
filter = ""

[dicts.source]
values = ["a", "b"]
"#;

fn project() -> tempfile::TempDir {
    let dir = scaffold_project(CONFIG);
    write_file(dir.path(), "pack.toml", PACK);
    dir
}

fn stdout(dir: &std::path::Path, args: &[&str]) -> String {
    let out = mt(dir).args(args).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    String::from_utf8(out.stdout).unwrap()
}

fn json(dir: &std::path::Path, args: &[&str]) -> serde_json::Value {
    serde_json::from_str(&stdout(dir, args)).unwrap()
}

/// The bare index names every listing, grouped, and puts `default` first in
/// the view list although the config declares it last.
#[test]
fn bare_lists_every_listing_grouped() {
    let dir = project();
    let text = stdout(dir.path(), &["explain"]);
    for part in [
        "rules:\n",
        "  tasks.rules: verb-title, [1]\n",
        "  tasks.body.headers.rules: one-problem\n",
        "  commits.rules: names-a-task\n",
        "  on.flip: [0]\n",
        "derived fields: changes\n",
        "tags: default, cli, mt\n",
        "views: default, blocked\n",
        "dicts: mindtape-dev-verbs (",
        "source (2)",
    ] {
        assert!(text.contains(part), "missing {part:?} in:\n{text}");
    }
    assert!(
        !text.contains("tasks.body.content.rules:"),
        "a section with no rule is dropped:\n{text}"
    );
}

/// The index ends on the first command to run, and the JSON envelope carries
/// no prose line.
#[test]
fn bare_ends_on_the_next_command() {
    let dir = project();
    let text = stdout(dir.path(), &["explain"]);
    assert!(
        text.ends_with("\nnext: run mt ls for your work; mt explain <rules|tags|views|dicts> opens one category, and --json prints a listing as data\n"),
        "{text}"
    );
    let value = json(dir.path(), &["explain", "--json"]);
    assert!(value.get("next").is_none(), "{value}");
}

/// The index JSON carries the same groups under a `schema_version`, while each
/// listing keeps its own envelope.
#[test]
fn json_index_groups_and_listings_keep_their_keys() {
    let dir = project();
    let index = json(dir.path(), &["explain", "--json"]);
    assert_eq!(index["schema_version"], mindtape::SCHEMA_VERSION);
    assert_eq!(
        index["rules"]["tasks.rules"],
        serde_json::json!(["verb-title", "[1]"])
    );
    assert_eq!(index["rules"]["on.flip"], serde_json::json!(["[0]"]));
    assert_eq!(index["derived"], serde_json::json!(["changes"]));
    assert_eq!(index["tags"], serde_json::json!(["default", "cli", "mt"]));
    assert_eq!(index["views"], serde_json::json!(["default", "blocked"]));
    assert_eq!(index["dicts"]["source"], 2);

    let rules = json(dir.path(), &["explain", "rules", "--json"]);
    assert!(rules["rules"].is_array() && rules["derived"].is_array());
    let tags = json(dir.path(), &["explain", "tags", "--json"]);
    assert!(tags["tags"].is_object());
}

/// A global `--json` reads the same before and after the listing.
#[test]
fn json_flag_parses_before_and_after_the_listing() {
    let dir = project();
    let before = stdout(dir.path(), &["explain", "--json", "views"]);
    let after = stdout(dir.path(), &["explain", "views", "--json"]);
    assert_eq!(before, after);
}

/// The task and body rules, padded to the widest name of the full list.
const TASK_LINES: &str = "\
error  tasks.rules.verb-title                open the title with an imperative verb
warn   tasks.rules[1]                        must satisfy `has(title)`
error  tasks.body.headers.rules.one-problem  keep one Problem section
";

const COMMIT_LINE: &str =
    "warn   commits.rules.names-a-task            name the task in a Refs: footer\n";

const FLIP_LINE: &str =
    "error  on.flip[0].actions[0]                 run the tests, then flip again\n";

#[test]
fn rules_lists_each_rule_after_the_pack_override_and_disable() {
    let dir = project();
    let text = stdout(dir.path(), &["explain", "rules"]);
    assert!(
        text.starts_with(&format!("{TASK_LINES}{COMMIT_LINE}{FLIP_LINE}")),
        "{text}"
    );
    assert!(text.contains("derived fields\n"), "{text}");
    assert!(text.contains("[path]  derived.changes"), "{text}");
}

/// Each array rule is named as `mt config show --origin` names it, in its
/// order, so a reader can look a listed rule up there.
#[test]
fn rules_names_each_rule_as_config_show_origin_does() {
    let dir = project();
    let origin = stdout(dir.path(), &["config", "show", "--origin"]);
    let labels: Vec<&str> = origin
        .lines()
        .filter_map(|line| line.strip_prefix("rule "))
        .filter_map(|line| line.split(": ").next())
        .filter(|label| {
            ["tasks.", "commits.", "on.flip"]
                .iter()
                .any(|prefix| label.starts_with(prefix))
        })
        .collect();
    let listed = stdout(dir.path(), &["explain", "rules"]);
    let names: Vec<&str> = listed
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .map(|name| name.split_once(".actions[").map_or(name, |(hook, _)| hook))
        .filter(|name| {
            ["tasks.", "commits.", "on.flip"]
                .iter()
                .any(|prefix| name.starts_with(prefix))
        })
        .collect();
    assert_eq!(names, labels);
}

/// `mt add` checks the task and body rules, `mt flip` those plus its flip
/// actions, and a commit only the commit rules.
#[test]
fn rules_for_keeps_the_rules_that_moment_checks() {
    let dir = project();
    for (moment, expected) in [
        ("add", TASK_LINES.to_owned()),
        ("flip", format!("{TASK_LINES}{FLIP_LINE}")),
        (
            "commit",
            "warn   commits.rules.names-a-task  name the task in a Refs: footer\n".to_owned(),
        ),
    ] {
        let text = stdout(dir.path(), &["explain", "rules", "--for", moment]);
        assert!(text.starts_with(&expected), "--for {moment}: {text}");
    }
}

#[test]
fn rules_json_carries_the_section_when_and_assert() {
    let dir = project();
    let value = json(dir.path(), &["explain", "rules", "--json"]);
    assert_eq!(value["schema_version"], mindtape::SCHEMA_VERSION);
    let rules = value["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 5, "{value}");
    assert_eq!(
        rules[0],
        serde_json::json!({
            "section": "tasks.rules",
            "name": "tasks.rules.verb-title",
            "severity": "error",
            "message": "open the title with an imperative verb",
            "when": "!terminal",
            "assert": "has(title)",
            "check": null,
        })
    );
    assert_eq!(rules[4]["name"], "on.flip[0].actions[0]");
}

#[test]
fn rules_says_so_when_no_rule_applies() {
    let dir = scaffold_project("[tasks]\nroots = [\"tasks\"]\n");
    assert_eq!(
        stdout(dir.path(), &["explain", "rules"]),
        "no rules apply\n"
    );
}

#[test]
fn rules_refuses_an_unknown_moment() {
    let dir = project();
    let (ok, text) = run_text(dir.path(), &["explain", "rules", "--for", "push"]);
    assert!(!ok, "{text}");
    assert!(
        text.contains("possible values: add, commit, flip"),
        "{text}"
    );
}

/// The tag scan: an allowed tag with the rules that read it and their
/// messages, an area label for one no rule reads, and `--for` narrowing both.
#[test]
fn tags_lists_each_allowed_tag_and_its_rules() {
    let dir = scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [tasks.tags]\nallowed = [\"bug\", \"cli\"]\n\n\
         [[tasks.rules]]\nname = \"bugs-need-repro\"\n\
         when = 'has_tag(\"bug\")'\nassert = 'has(assignee)'\nmessage = \"show a repro\"\n",
    );
    assert_eq!(
        stdout(dir.path(), &["explain", "tags"]),
        "\
bug  tasks.rules.bugs-need-repro
     show a repro
cli  area label
"
    );
    assert_eq!(
        stdout(dir.path(), &["explain", "tags", "--for", "commit"]),
        "no tags are read at this moment\n"
    );
    let value = json(dir.path(), &["explain", "tags", "--json"]);
    assert_eq!(
        value["tags"]["bug"],
        serde_json::json!([{"rule": "tasks.rules.bugs-need-repro", "message": "show a repro"}])
    );
    assert_eq!(value["tags"]["cli"], serde_json::json!([]));
}

/// `mt explain views` lists the declared views and shows one in full.
#[test]
fn views_lists_and_inspects_a_named_view() {
    let dir = scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [views.hot]\nfilter = 'priority > 5'\nsort = ['priority:desc']\n\
         [views.simple]\nfilter = ''\n",
    );
    let listed = stdout(dir.path(), &["explain", "views"]);
    assert!(
        listed.contains("hot") && listed.contains("simple"),
        "{listed}"
    );

    let detail = stdout(dir.path(), &["explain", "views", "hot"]);
    assert!(detail.contains("priority > 5"), "{detail}");
    assert!(detail.contains("sort:"), "{detail}");
    assert!(detail.contains("priority:desc"), "{detail}");

    let simple = stdout(dir.path(), &["explain", "views", "simple"]);
    assert!(!simple.contains("sort:"), "{simple}");
    assert!(!simple.contains("cols:"), "{simple}");
}

#[test]
fn views_unknown_name_suggests_the_near_miss() {
    let dir = scaffold_project("[tasks]\nroots = [\"tasks\"]\n\n[views.hot]\nfilter = ''\n");
    let (ok, text) = run_text(dir.path(), &["explain", "views", "hott"]);
    assert!(!ok, "{text}");
    assert!(
        text.contains("error: unknown view: 'hott'\nhint: did you mean hot?\n"),
        "{text}"
    );
}

/// The dictionary list, one dictionary's entries, and the refusal that names
/// the command which lists them.
#[test]
fn dicts_lists_counts_and_entries() {
    let dir = scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [dicts.verbs]\nvalues = [\"vote\", \"Rose\", \"rotate\"]\n\n\
         [dicts.labels]\nvalues = [\"BILLING\", \"Auth\"]\ncase = \"exact\"\n",
    );
    let listed = stdout(dir.path(), &["explain", "dicts"]);
    assert!(listed.contains("labels"), "{listed}");
    assert!(listed.contains("2 entries, case exact"), "{listed}");
    assert!(listed.contains("3 entries, case fold"), "{listed}");

    assert_eq!(
        stdout(dir.path(), &["explain", "dicts", "labels"]),
        "Auth\nBILLING\n"
    );
    let listed = json(dir.path(), &["explain", "dicts", "--json"]);
    assert_eq!(
        listed["labels"],
        serde_json::json!({"case": "exact", "count": 2})
    );
    assert_eq!(
        listed["verbs"],
        serde_json::json!({"case": "fold", "count": 3})
    );
    let value = json(dir.path(), &["explain", "dicts", "labels", "--json"]);
    assert_eq!(
        value,
        serde_json::json!({
            "name": "labels", "case": "exact", "count": 2, "entries": ["Auth", "BILLING"],
        })
    );

    let (ok, text) = run_text(dir.path(), &["explain", "dicts", "verbz"]);
    assert!(!ok, "{text}");
    assert!(text.contains("hint: did you mean verbs?"), "{text}");
    assert!(
        text.contains("hint: use one of: labels, mindtape-dev-verbs, mindtape-verbs, verbs"),
        "{text}"
    );
}

/// The replacement is whole: no old spelling survives as an alias.
#[test]
fn the_four_old_commands_are_gone() {
    let dir = project();
    for command in ["rules", "tags", "views", "dicts"] {
        let (ok, text) = run_text(dir.path(), &[command]);
        assert!(!ok, "`mt {command}` must not parse: {text}");
        assert!(text.contains("unknown command"), "{text}");
    }
}
