#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use typst_harvest::HVal;

use super::{CountEstimate, Query, annotate, parse_estimate, validate_config};
use crate::config::parse_config;
use crate::test_support::{bare_task, git_init, tempdir, write_file};

fn dict(pairs: &[(&str, HVal)]) -> HVal {
    HVal::Dict(
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.clone()))
            .collect(),
    )
}

fn text(value: &str) -> HVal {
    HVal::Str(value.to_owned())
}

fn grep(regex: &str, within: Option<&str>) -> HVal {
    dict(&[
        ("query", text("grep")),
        ("regex", text(regex)),
        ("within", within.map_or(HVal::Null, text)),
    ])
}

fn count(manual: i64, queries: Vec<HVal>) -> HVal {
    dict(&[
        ("count", HVal::Int(manual)),
        ("queries", HVal::Array(queries)),
    ])
}

#[test]
fn a_count_unit_parses_beside_a_plain_one() {
    let value = dict(&[
        (
            "src-files",
            count(
                2,
                vec![
                    grep("fn", Some("src/**")),
                    dict(&[("query", text("glob")), ("pattern", text("docs/*"))]),
                ],
            ),
        ),
        ("hours", HVal::Int(3)),
    ]);
    let mut errors = Vec::new();
    let (plain, counts) = parse_estimate(Some(&value), &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(plain, BTreeMap::from([("hours".to_owned(), 3.0)]));
    let expected = CountEstimate {
        manual: 2.0,
        queries: vec![
            Query::Grep {
                regex: "fn".to_owned(),
                within: Some("src/**".to_owned()),
            },
            Query::Glob {
                pattern: "docs/*".to_owned(),
            },
        ],
        problem: None,
    };
    assert_eq!(counts.get("src-files"), Some(&expected));
}

#[test]
fn a_query_that_is_no_query_is_a_shape_error_at_its_index() {
    let value = dict(&[("src-files", count(1, vec![text("fn")]))]);
    let mut errors = Vec::new();
    let (_, counts) = parse_estimate(Some(&value), &mut errors);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        errors.first().unwrap().field,
        "estimate.src-files.queries[0]"
    );
    assert!(counts.get("src-files").unwrap().queries.is_empty());
}

#[test]
fn a_count_without_a_number_is_a_shape_error() {
    let value = dict(&[("src-files", dict(&[("count", text("two"))]))]);
    let mut errors = Vec::new();
    let (plain, counts) = parse_estimate(Some(&value), &mut errors);
    assert_eq!(errors.first().unwrap().field, "estimate.src-files");
    assert!(plain.is_empty() && counts.is_empty());
}

#[test]
fn a_queries_unit_outside_the_allow_list_is_a_config_problem() {
    let config = parse_config(
        "[tasks.estimate]\nunits = [\"files\"]\n\n[tasks.estimate.queries]\nfiles = \"src/**\"\nhours = \"src/**\"\n",
    )
    .unwrap();
    assert_eq!(
        validate_config(&config),
        vec!["tasks.estimate.queries.hours: `hours` is not an allowed estimate unit".to_owned()]
    );
    let config = parse_config("[tasks.estimate.queries]\nfiles = \"src/**\"\n").unwrap();
    assert!(validate_config(&config).is_empty());
}

/// A task estimating `hours: 2` and `files: count(1, query)`, resolved under
/// `dir` with `files` counting inside `src/**`.
fn annotated(dir: &std::path::Path, query: Query) -> crate::model::Task {
    let config = parse_config("[tasks.estimate.queries]\nfiles = \"src/**\"\n").unwrap();
    let mut task = bare_task("t");
    task.estimate.insert("hours".to_owned(), 2.0);
    let count = CountEstimate {
        manual: 1.0,
        queries: vec![query],
        problem: None,
    };
    task.estimate_counts.insert("files".to_owned(), count);
    let mut tasks = vec![task];
    annotate(&mut tasks, &config, dir);
    tasks.pop().unwrap()
}

#[test]
fn annotate_writes_the_resolved_count_and_clears_nothing_else() {
    let dir = tempdir();
    git_init(dir.path());
    write_file(dir.path(), "src/a.rs", "fn parse() {}\n");
    write_file(dir.path(), "src/b.rs", "fn other() {}\n");
    write_file(dir.path(), "src/c.rs", "const C: u8 = 1;\n");
    write_file(dir.path(), "docs/a.md", "fn parse()\n");
    let regex = r"fn \w+\(".to_owned();
    let task = annotated(
        dir.path(),
        Query::Grep {
            regex,
            within: None,
        },
    );
    assert_eq!(task.estimate.get("files"), Some(&3.0));
    assert_eq!(task.estimate.get("hours"), Some(&2.0));
    assert_eq!(task.estimate_counts.get("files").unwrap().problem, None);
}

#[test]
fn annotate_outside_git_records_why_and_leaves_the_unit_out() {
    let dir = tempdir();
    let pattern = "src/*".to_owned();
    let task = annotated(dir.path(), Query::Glob { pattern });
    assert!(!task.estimate.contains_key("files"));
    let count = task.estimate_counts.get("files").unwrap();
    let problem = count.problem.clone().unwrap();
    assert!(
        problem.starts_with("can't list files for a query:"),
        "{problem}"
    );
}
