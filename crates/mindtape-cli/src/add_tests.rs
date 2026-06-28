//! Unit tests for the `add` command's pure helpers.
//!
//! Binary-invoking end-to-end tests live in `tests/add_e2e.rs` (integration
//! tests), where Cargo guarantees the `mt` binary is built first.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::Path;

use mindtape::{Config, RawValue};

use super::{AddArgs, dest_dir, fill_title_from_args, initial_status, inject_header, output_stem};
use crate::util::Project;

const HEADER: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Do thing",
)

== Summary
"#;

fn bare_args() -> AddArgs {
    AddArgs {
        title: vec![],
        template: None,
        values: vec![],
        set: vec![],
        into: ".".to_owned(),
        to: None,
        name: None,
        status: None,
        edit: false,
        describe: None,
        list: false,
        print: false,
        no_verify: false,
    }
}

#[test]
fn injects_default_status_proposed() {
    let out = inject_header(HEADER, &Config::default(), None).unwrap();
    assert!(out.contains("status: proposed("), "{out}");
    assert!(!out.contains("status(\"proposed\""), "{out}");
}

#[test]
fn status_override_wins() {
    let out = inject_header(HEADER, &Config::default(), Some("wip")).unwrap();
    assert!(out.contains("status: wip("), "{out}");
}

#[test]
fn custom_status_falls_back_to_verbose_form() {
    // A name the prelude ships no shortcut for keeps the `status("…", …)` form.
    let out = inject_header(HEADER, &Config::default(), Some("icebox")).unwrap();
    assert!(out.contains("status: status(\"icebox\","), "{out}");
}

#[test]
fn initial_status_is_first_non_terminal() {
    let config: Config = mindtape::parse_config(
        "[[tasks.status]]\nvalue = \"done\"\nterminal = true\n\
         [[tasks.status]]\nvalue = \"open\"\n",
    )
    .unwrap();
    assert_eq!(initial_status(&config).as_deref(), Some("open"));
}

#[test]
fn output_stem_prefers_name_then_title_slug() {
    let mut args = bare_args();
    args.name = Some("explicit".to_owned());
    assert_eq!(output_stem(&args, &BTreeMap::new()).unwrap(), "explicit");

    let args = bare_args();
    let mut values = BTreeMap::new();
    values.insert("title".to_owned(), RawValue::Str("My New Task".to_owned()));
    assert_eq!(output_stem(&args, &values).unwrap(), "my-new-task");
}

#[test]
fn title_arg_joins_words_when_unset() {
    let mut args = bare_args();
    args.title = vec!["cleanup".to_owned(), "kitchen".to_owned()];
    let mut values = BTreeMap::new();
    fill_title_from_args(&mut values, &args);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("cleanup kitchen".to_owned()))
    );
}

#[test]
fn title_arg_yields_to_higher_precedence_title() {
    let mut args = bare_args();
    args.title = vec!["positional".to_owned()];
    let mut values = BTreeMap::new();
    values.insert("title".to_owned(), RawValue::Str("explicit".to_owned()));
    fill_title_from_args(&mut values, &args);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("explicit".to_owned()))
    );
}

#[test]
fn output_stem_errors_without_name_or_title() {
    assert!(output_stem(&bare_args(), &BTreeMap::new()).is_err());
}

/// A `Project` rooted at `dir` whose single configured root is `root`.
fn project_with_root(dir: &Path, root: &str) -> Project {
    let mut config = Config::default();
    config.tasks.roots = vec![mindtape::RootSpec::Short(root.to_owned())];
    Project {
        root: dir.to_path_buf(),
        config,
        outcome: mindtape::LoadOutcome::default(),
    }
}

#[test]
fn dest_default_is_first_root() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_with_root(dir.path(), "truth/tasks");
    assert_eq!(
        dest_dir(".", None, &project).unwrap(),
        dir.path().join("truth/tasks")
    );
}

#[test]
fn dest_to_nests_subfolder_under_root() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_with_root(dir.path(), "truth/tasks");
    assert_eq!(
        dest_dir(".", Some("shopping"), &project).unwrap(),
        dir.path().join("truth/tasks/shopping")
    );
    // Nested subpaths are allowed.
    assert_eq!(
        dest_dir(".", Some("shopping/weekly"), &project).unwrap(),
        dir.path().join("truth/tasks/shopping/weekly")
    );
}

#[test]
fn dest_to_rejects_absolute_subfolder() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_with_root(dir.path(), "tasks");
    assert!(dest_dir(".", Some("/etc"), &project).is_err());
}
