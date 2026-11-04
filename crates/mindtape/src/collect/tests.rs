#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use typst_harvest::HVal;

use crate::config::{
    CollectorAtom, CollectorSpec, CollectorType, Config, RegexQuery, SectionSpec, parse_config,
};
use crate::filter::Filter;
use crate::model::Task;
use crate::sort::{sort_by_specs, validate_sort_spec_with};

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

#[test]
fn a_declared_collector_may_be_named_in_its_own_when() {
    let spec = CollectorSpec {
        name: "changes".to_owned(),
        when: Some("has(derived.changes)".to_owned()),
        query: crate::config::CollectorQuery::Regex(RegexQuery {
            section: SectionSpec::Name("Changes".to_owned()),
            pattern: "x".to_owned(),
            group: None,
        }),
        returns: CollectorType::List(Box::new(CollectorType::Atom(CollectorAtom::Path))),
    };
    let mut config = Config::default();
    config.tasks.collectors.push(spec);
    assert!(config.validate().is_ok(), "{:?}", config.validate());
}
