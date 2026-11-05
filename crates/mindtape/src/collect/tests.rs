#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use typst_harvest::HVal;

use crate::config::layers::load_config_layered_from;
use crate::config::{
    CollectorAtom, CollectorSpec, CollectorType, Config, RegexQuery, SectionSpec, parse_config,
};
use crate::filter::Filter;
use crate::model::Task;
use crate::sort::{sort_by_specs, validate_sort_spec_with};
use crate::test_support::{git_tempdir, tempdir, write_file};
use crate::validate::{ViolationKind, validate};

fn config(text: &str) -> Config {
    parse_config(text).unwrap()
}

const CHANGES: &str = r#"
[[tasks.collectors]]
name = "changes"
when = "!terminal"
query = { section = "Changes", pattern = "(?m)^\\s*-\\s+`([^`]+)`" }
returns = ["path"]
"#;

fn task(body: &str) -> Task {
    Task {
        id: "t".to_owned(),
        body: body.to_owned(),
        ..Task::default()
    }
}

fn strings(task: &Task, name: &str) -> Vec<String> {
    task.derived[name]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn a_regex_collector_takes_the_first_capture_group() {
    let config = config(CHANGES);
    let mut tasks = vec![task("== Changes\n\n- `src/a.rs`\n- `src/b.rs`\n")];
    crate::collect::annotate(&mut tasks, &config);
    assert_eq!(strings(&tasks[0], "changes"), ["src/a.rs", "src/b.rs"]);
}

#[test]
fn a_pattern_without_a_group_takes_the_whole_match() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "files"
query = { section = "Changes", pattern = "src/[a-z]+\\.rs" }
returns = ["string"]
"#,
    );
    let mut tasks = vec![task("== Changes\n\n- src/a.rs and src/b.rs\n")];
    crate::collect::annotate(&mut tasks, &config);
    assert_eq!(strings(&tasks[0], "files"), ["src/a.rs", "src/b.rs"]);
}

#[test]
fn group_zero_is_the_whole_match() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "files"
query = { section = "Changes", pattern = "src/([a-z]+)\\.rs", group = 0 }
returns = ["string"]
"#,
    );
    let mut tasks = vec![task("== Changes\n\n- src/a.rs\n")];
    crate::collect::annotate(&mut tasks, &config);
    assert_eq!(strings(&tasks[0], "files"), ["src/a.rs"]);
}

#[test]
fn a_missing_section_is_an_empty_list() {
    let config = config(CHANGES);
    let mut tasks = vec![task("== Summary\n\nnothing here\n")];
    crate::collect::annotate(&mut tasks, &config);
    assert_eq!(strings(&tasks[0], "changes"), Vec::<String>::new());
}

#[test]
fn a_gated_task_carries_no_field() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "changes"
when = "status == \"done\""
query = { section = "Changes", pattern = "(?m)^\\s*-\\s+`([^`]+)`" }
returns = ["path"]
"#,
    );
    let mut tasks = vec![task("== Changes\n\n- `src/a.rs`\n")];
    crate::collect::annotate(&mut tasks, &config);
    assert!(!tasks[0].derived.contains_key("changes"));
}

#[test]
fn the_section_slice_stops_at_the_next_heading() {
    let config = config(CHANGES);
    let mut tasks = vec![task(
        "== Changes\n\n- `src/a.rs`\n\n== TO-BE\n\n- `src/b.rs`\n",
    )];
    crate::collect::annotate(&mut tasks, &config);
    assert_eq!(strings(&tasks[0], "changes"), ["src/a.rs"]);
}

#[test]
fn len_counts_a_list_and_count_paths_counts_distinct_normalized_paths() {
    let config = config(&format!(
        "{CHANGES}\n[dicts.source]\ncase = \"exact\"\nvalues = [\"src/**\"]\n"
    ));
    let mut tasks = vec![task(
        "== Changes\n\n- `./src/a.rs`\n- `src/a.rs`\n- `docs/b.md`\n",
    )];
    crate::collect::annotate(&mut tasks, &config);
    let task = &tasks[0];
    assert!(
        Filter::parse("len(derived.changes) == 3")
            .unwrap()
            .matches(task, &config)
            .unwrap()
    );
    assert!(
        Filter::parse("count_paths(derived.changes, \"@source\") == 1")
            .unwrap()
            .matches(task, &config)
            .unwrap()
    );
}

#[test]
fn a_record_field_reads_as_a_scalar() {
    let mut config = config(CHANGES);
    config.tasks.collectors[0].returns = CollectorType::Record(BTreeMap::from([(
        "count".to_owned(),
        CollectorType::Atom(CollectorAtom::Number),
    )]));
    let mut task = task("");
    task.derived.insert(
        "changes".to_owned(),
        HVal::Dict(BTreeMap::from([("count".to_owned(), HVal::Int(3))])),
    );
    assert!(
        Filter::parse("derived.changes.count > 2")
            .unwrap()
            .matches(&task, &config)
            .unwrap()
    );
    assert_eq!(
        crate::model::fields::derived(&task, "derived.changes.count", &config),
        Some(crate::model::fields::FieldValue::Num(3.0))
    );
}

#[test]
fn comparing_a_list_or_record_fails() {
    let config = config(CHANGES);
    let mut task = task("");
    task.derived.insert(
        "changes".to_owned(),
        HVal::Array(vec![HVal::Str("src/a.rs".to_owned())]),
    );
    let err = Filter::parse("derived.changes == \"src/a.rs\"")
        .unwrap()
        .matches(&task, &config)
        .unwrap_err();
    assert!(err.to_string().contains("use len()"), "{err}");
}

#[test]
fn only_a_scalar_derived_field_sorts() {
    let mut config = config(CHANGES);
    let problem = validate_sort_spec_with("derived.changes", &config.tasks.collectors).unwrap_err();
    assert!(problem.text.contains("only a scalar sorts"), "{problem}");
    config.tasks.collectors[0].returns = CollectorType::Record(BTreeMap::from([(
        "count".to_owned(),
        CollectorType::Atom(CollectorAtom::Number),
    )]));
    assert!(validate_sort_spec_with("derived.changes.count", &config.tasks.collectors).is_ok());
    let mut first = task("");
    first.id = "a".to_owned();
    first.derived.insert(
        "changes".to_owned(),
        HVal::Dict(BTreeMap::from([("count".to_owned(), HVal::Int(1))])),
    );
    let mut second = task("");
    second.id = "b".to_owned();
    second.derived.insert(
        "changes".to_owned(),
        HVal::Dict(BTreeMap::from([("count".to_owned(), HVal::Int(2))])),
    );
    let mut tasks = vec![second, first];
    sort_by_specs(
        &mut tasks,
        &["derived.changes.count".to_owned()],
        &config,
        &crate::Readiness::empty(),
    )
    .unwrap();
    assert_eq!(tasks[0].id, "a");
}

#[test]
fn a_missing_derived_field_is_missing_not_unknown() {
    let config = config(CHANGES);
    let task = task("");
    assert!(
        Filter::parse("miss(derived.changes)")
            .unwrap()
            .matches(&task, &config)
            .unwrap()
    );
    assert!(
        Filter::parse("len(derived.changes) == 0")
            .unwrap()
            .matches(&task, &config)
            .unwrap()
    );
}

#[test]
fn an_undeclared_derived_field_is_unknown() {
    let config = config(CHANGES);
    let task = task("");
    let err = Filter::parse("has(derived.nope)")
        .unwrap()
        .matches(&task, &config)
        .unwrap_err();
    assert!(err.to_string().contains("derived.nope"), "{err}");
}

const CALLERS: &str = r#"
[tasks]
roots = ["tasks"]

[[tasks.collectors]]
name = "callers"
query = { ast_grep = { language = "rust", pattern = "parse_task($$$ARGS)" } }
returns = ["path"]
"#;

/// Load `config` from a git repository holding `files`, so the shared query
/// engine has a working tree to list.
fn ast_project(config: &str, files: &[(&str, &str)]) -> (tempfile::TempDir, Config) {
    let dir = git_tempdir();
    for (path, contents) in files {
        write_file(dir.path(), path, contents);
    }
    write_file(dir.path(), ".mindtape/config.toml", config);
    let config = load_config_layered_from(dir.path(), Some(dir.path()), None)
        .unwrap()
        .0;
    (dir, config)
}

/// The one task `config` annotates over a git project holding `files`.
fn collected(config: &str, files: &[(&str, &str)]) -> Task {
    let (_dir, config) = ast_project(config, files);
    let mut tasks = vec![task("")];
    crate::collect::annotate(&mut tasks, &config);
    let task = tasks.pop().unwrap();
    assert!(
        task.collector_problems.is_empty(),
        "{:?}",
        task.collector_problems
    );
    task
}

/// `source` under one Rust name and two names the Rust grammar must skip.
fn rust_and_decoys(source: &str) -> [(&str, &str); 3] {
    [
        ("src/a.rs", source),
        ("src/notes.md", source),
        ("tasks/t.typ", source),
    ]
}

#[test]
fn an_ast_grep_collector_skips_files_outside_the_languages_extensions() {
    let task = collected(CALLERS, &rust_and_decoys("fn a() { parse_task(1); }\n"));
    assert_eq!(strings(&task, "callers"), ["src/a.rs"]);
}

#[test]
fn an_ast_grep_collector_publishes_each_matching_path() {
    let task = collected(
        CALLERS,
        &[
            ("src/a.rs", "fn a() { parse_task(1); }\n"),
            ("src/b.rs", "fn b() { other(2); }\n"),
            ("docs/c.rs", "fn c() { parse_task(3); }\n"),
        ],
    );
    assert_eq!(strings(&task, "callers"), ["docs/c.rs", "src/a.rs"]);
}

#[test]
fn an_ast_grep_glob_narrows_the_collected_paths() {
    let config = CALLERS.replace(
        r#"pattern = "parse_task($$$ARGS)" }"#,
        r#"pattern = "parse_task($$$ARGS)", glob = "src/**" }"#,
    );
    let task = collected(
        &config,
        &[
            ("src/a.rs", "fn a() { parse_task(1); }\n"),
            ("docs/c.rs", "fn c() { parse_task(3); }\n"),
        ],
    );
    assert_eq!(strings(&task, "callers"), ["src/a.rs"]);
}

#[test]
fn a_query_the_engine_cannot_run_is_a_collector_problem() {
    let dir = tempdir();
    write_file(dir.path(), ".mindtape/config.toml", CALLERS);
    let config = load_config_layered_from(dir.path(), Some(dir.path()), None)
        .unwrap()
        .0;
    let mut tasks = vec![task("")];
    crate::collect::annotate(&mut tasks, &config);
    assert!(!tasks[0].derived.contains_key("callers"));
    let problem = tasks[0].collector_problems[0].text.clone();
    assert!(problem.contains("derived.callers"), "{problem}");
    let violations = validate(&tasks, &config);
    let violation = violations
        .iter()
        .find(|violation| violation.kind == ViolationKind::Collector)
        .unwrap();
    assert_eq!(violation.field, "derived.callers");
    assert!(violation.is_fatal());
}

/// A regex collector over Changes, for tests that only need a valid spec.
fn regex_spec(name: &str, when: Option<&str>) -> CollectorSpec {
    CollectorSpec {
        name: name.to_owned(),
        when: when.map(str::to_owned),
        query: crate::config::CollectorQuery::Regex(RegexQuery {
            section: SectionSpec::Name("Changes".to_owned()),
            pattern: "x".to_owned(),
            group: None,
        }),
        returns: CollectorType::List(Box::new(CollectorType::Atom(CollectorAtom::Path))),
        root: None,
    }
}

#[test]
fn a_collector_cannot_read_its_own_field_in_its_when() {
    let mut config = Config::default();
    config
        .tasks
        .collectors
        .push(regex_spec("changes", Some("has(derived.changes)")));
    let problem = config.validate().unwrap_err();
    assert!(problem.text.contains("declaration order"), "{problem}");
}

#[test]
fn a_later_collector_may_read_an_earlier_field_but_not_the_reverse() {
    let mut config = Config::default();
    config.tasks.collectors.push(regex_spec("changes", None));
    config
        .tasks
        .collectors
        .push(regex_spec("summary", Some("len(derived.changes) > 0")));
    assert!(config.validate().is_ok(), "{:?}", config.validate());
    let mut reversed = config.clone();
    reversed.tasks.collectors.reverse();
    let problem = reversed.validate().unwrap_err();
    assert!(problem.text.contains("declaration order"), "{problem}");
}

#[cfg(unix)]
#[test]
fn a_collector_publishes_one_path_per_lossy_spelling() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let (dir, config) = ast_project(CALLERS, &[]);
    for name in [b"bad\xffname.rs".as_slice(), b"bad\xfename.rs".as_slice()] {
        let path = std::path::Path::new("src").join(OsStr::from_bytes(name));
        write_file(dir.path(), &path, "fn a() { parse_task(1); }\n");
    }
    let mut tasks = vec![task("")];
    crate::collect::annotate(&mut tasks, &config);
    assert_eq!(strings(&tasks[0], "callers").len(), 1);
}

/// `problem` names a missing `@sorce` and carries the nearest `@source`.
fn assert_nearest_source(problem: &crate::problem::Problem) {
    assert!(problem.text.contains("no path set"), "{problem}");
    assert_eq!(
        problem.unknown.as_ref().unwrap().nearest.as_deref(),
        Some("@source")
    );
}

#[test]
fn a_collector_keeps_the_nearest_path_set_name_as_a_hint() {
    let config = format!("{CALLERS}\n[dicts.source]\ncase = \"exact\"\nvalues = [\"src/**\"]\n")
        .replace(
            r#"pattern = "parse_task($$$ARGS)" }"#,
            r#"pattern = "parse_task($$$ARGS)", glob = "@sorce" }"#,
        );
    let dir = git_tempdir();
    write_file(dir.path(), "src/a.rs", "fn a() { parse_task(1); }\n");
    let mut config = parse_config(&config).unwrap();
    config.rules_root.set(dir.path());
    assert_nearest_source(&config.validate().unwrap_err());
    let mut tasks = vec![task("")];
    crate::collect::annotate(&mut tasks, &config);
    assert_nearest_source(&tasks[0].collector_problems[0]);
}
