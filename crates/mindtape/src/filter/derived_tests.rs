#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use typst_harvest::HVal;

use crate::config::{CollectorAtom, CollectorSpec, CollectorType, Config, RegexQuery, SectionSpec};
use crate::filter::Filter;
use crate::model::Task;

fn config() -> Config {
    let mut config = Config::default();
    config.tasks.collectors.push(CollectorSpec {
        name: "changes".to_owned(),
        when: None,
        query: crate::config::CollectorQuery::Regex(RegexQuery {
            section: SectionSpec::Name("Changes".to_owned()),
            pattern: "x".to_owned(),
            group: None,
        }),
        returns: CollectorType::List(Box::new(CollectorType::Atom(CollectorAtom::Path))),
        root: None,
    });
    config.tasks.collectors.push(CollectorSpec {
        name: "summary".to_owned(),
        when: None,
        query: crate::config::CollectorQuery::Regex(RegexQuery {
            section: SectionSpec::Name("Changes".to_owned()),
            pattern: "x".to_owned(),
            group: None,
        }),
        returns: CollectorType::Record(BTreeMap::from([
            (
                "count".to_owned(),
                CollectorType::Atom(CollectorAtom::Number),
            ),
            (
                "name".to_owned(),
                CollectorType::Atom(CollectorAtom::String),
            ),
        ])),
        root: None,
    });
    config
}

fn subject_with(derived: BTreeMap<String, HVal>) -> Task {
    Task {
        id: "t".to_owned(),
        derived,
        ..Task::default()
    }
}

fn changes(paths: &[&str]) -> BTreeMap<String, HVal> {
    BTreeMap::from([(
        "changes".to_owned(),
        HVal::Array(
            paths
                .iter()
                .map(|path| HVal::Str((*path).to_owned()))
                .collect(),
        ),
    )])
}

fn matches(filter: &str, task: &Task, config: &Config) -> Result<bool, crate::FilterError> {
    Filter::parse(filter).unwrap().matches(task, config)
}

#[test]
fn len_reads_tags_a_derived_list_and_a_missing_field() {
    let config = config();
    let mut subject = subject_with(changes(&["src/a.rs", "src/b.rs"]));
    subject.tags = vec!["backend".to_owned()];
    assert!(matches("len(tags) == 1", &subject, &config).unwrap());
    assert!(matches("len(derived.changes) == 2", &subject, &config).unwrap());
    let empty = subject_with(BTreeMap::new());
    assert!(matches("len(derived.changes) == 0", &empty, &config).unwrap());
}

#[test]
fn len_refuses_a_non_list_and_a_wrong_arity() {
    let config = config();
    let subject = subject_with(changes(&["src/a.rs"]));
    assert!(matches("len(title) == 0", &subject, &config).is_err());
    assert!(matches("len(\"x\") == 0", &subject, &config).is_err());
    assert!(matches("len() == 0", &subject, &config).is_err());
}

#[test]
fn count_paths_reads_none_one_and_many_sets() {
    let config = crate::parse_config(
        r#"
[[tasks.collectors]]
name = "changes"
query = { section = "Changes", pattern = "x" }
returns = ["path"]

[dicts.source]
case = "exact"
values = ["src/**"]
"#,
    )
    .unwrap();
    let subject = subject_with(changes(&[
        "./src/a.rs",
        "src/a.rs",
        "src\\\\b.rs",
        "docs/c.md",
        "../src/d.rs",
    ]));
    assert!(matches("count_paths(derived.changes) == 4", &subject, &config).unwrap());
    assert!(
        matches(
            "count_paths(derived.changes, \"@source\") == 2",
            &subject,
            &config
        )
        .unwrap()
    );
    assert!(
        matches(
            "count_paths(derived.changes, \"docs/**\") == 1",
            &subject,
            &config
        )
        .unwrap()
    );
}

#[test]
fn count_paths_reads_zero_for_a_missing_field() {
    let config = config();
    let subject = subject_with(BTreeMap::new());
    assert!(matches("count_paths(derived.changes) == 0", &subject, &config).unwrap());
}

#[test]
fn count_paths_refuses_a_non_path_field_and_a_bad_argument() {
    let config = config();
    let subject = subject_with(changes(&["src/a.rs"]));
    assert!(matches("count_paths(derived.summary) == 0", &subject, &config).is_err());
    assert!(matches("count_paths(derived.nope) == 0", &subject, &config).is_err());
    assert!(matches("count_paths(\"x\") == 0", &subject, &config).is_err());
    assert!(matches("count_paths() == 0", &subject, &config).is_err());
    assert!(
        matches(
            "count_paths(derived.changes, other) == 0",
            &subject,
            &config
        )
        .is_err()
    );
}

#[test]
fn has_reads_a_list_a_record_a_scalar_and_a_missing_field() {
    let config = config();
    let mut subject = subject_with(changes(&["src/a.rs"]));
    subject.derived.insert(
        "summary".to_owned(),
        HVal::Dict(BTreeMap::from([
            ("count".to_owned(), HVal::Int(2)),
            ("name".to_owned(), HVal::Str("x".to_owned())),
        ])),
    );
    assert!(matches("has(derived.changes, \"src/a.rs\")", &subject, &config).unwrap());
    assert!(matches("has(derived.summary, \"count\")", &subject, &config).unwrap());
    assert!(matches("has(derived.summary.count)", &subject, &config).unwrap());
    assert!(matches("has(derived.missing)", &subject, &config).is_err());
    let gated = subject_with(BTreeMap::new());
    assert!(!matches("has(derived.changes)", &gated, &config).unwrap());
}

#[test]
fn comparing_a_record_fails() {
    let config = config();
    let mut subject = subject_with(BTreeMap::new());
    subject.derived.insert(
        "summary".to_owned(),
        HVal::Dict(BTreeMap::from([("count".to_owned(), HVal::Int(2))])),
    );
    let err = matches("derived.summary == \"x\"", &subject, &config).unwrap_err();
    assert!(err.to_string().contains("use len()"), "{err}");
    assert!(matches("derived.summary.count == 2", &subject, &config).unwrap());
}

#[test]
fn a_derived_name_is_a_term() {
    let config = config();
    assert!(Filter::names_term("derived.changes", &config));
    assert!(!Filter::names_term("derived.nope", &config));
}

#[test]
fn has_with_a_value_refuses_an_undeclared_derived_field() {
    let config = config();
    let subject = subject_with(BTreeMap::new());
    let err = matches("has(derived.nope, \"x\")", &subject, &config).unwrap_err();
    assert!(err.to_string().contains("derived.nope"), "{err}");
}
