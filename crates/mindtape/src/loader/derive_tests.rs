use std::path::Path;

use crate::config::{Collector, DeriveConfig, DeriveTags, RootConfig, RootSpec};
use crate::config::{Config, TasksConfig};

use super::{RootDeriver, validate_root_globs};

fn cfg(collect: Vec<Collector>) -> DeriveConfig {
    DeriveConfig {
        enabled: true,
        collect,
        ..DeriveConfig::default()
    }
}

fn root(name: &str) -> RootSpec {
    RootSpec::Full(RootConfig {
        path: "tasks".to_owned(),
        name: Some(name.to_owned()),
        derive_tags: None,
    })
}

/// The tags `[tasks.derive]` yields for `path` under a root named `mt`.
fn tags_for(global: &DeriveConfig, path: &str) -> Vec<String> {
    RootDeriver::new(&root("mt"), global).tags_for(Path::new(path))
}

/// Asserts the tags `[tasks.derive]` yields for `path` are exactly `expected`, in order.
fn assert_tags(global: &DeriveConfig, path: &str, expected: &[&str]) {
    assert_eq!(tags_for(global, path), expected, "tags for {path}");
}

#[test]
fn root_name_defaults_to_last_path_segment() {
    let spec = RootSpec::Short("truth/tasks".to_owned());
    assert_eq!(spec.name(), "tasks");
}

#[test]
fn disabled_global_yields_nothing() {
    let global = DeriveConfig::default(); // enabled = false
    assert!(tags_for(&global, "backend/auth/x.typ").is_empty());
}

#[test]
fn root_name_always_emitted_even_at_top_level() {
    let global = cfg(vec![Collector::Segments]);
    assert_tags(&global, "x.typ", &["mt"]);
}

#[test]
fn segments_emits_each_folder() {
    let global = cfg(vec![Collector::Segments]);
    assert_tags(&global, "backend/auth/x.typ", &["mt", "backend", "auth"]);
}

#[test]
fn breadcrumbs_joins_with_separator() {
    let global = cfg(vec![Collector::Breadcrumbs]);
    assert_tags(
        &global,
        "backend/auth/x.typ",
        &["mt", "backend", "backend:auth"],
    );
}

#[test]
fn both_collectors_dedupe() {
    let global = cfg(vec![Collector::Segments, Collector::Breadcrumbs]);
    assert_tags(
        &global,
        "backend/auth/x.typ",
        &["mt", "backend", "auth", "backend:auth"],
    );
}

#[test]
fn breadcrumb_root_folds_root_into_chain() {
    let global = DeriveConfig {
        enabled: true,
        collect: vec![Collector::Breadcrumbs],
        breadcrumb_root: true,
        ..DeriveConfig::default()
    };
    assert_tags(&global, "backend/x.typ", &["mt", "mt:backend"]);
}

#[test]
fn slugify_normalises_folder_names() {
    let global = cfg(vec![Collector::Segments]);
    assert_tags(&global, "Back End/x.typ", &["mt", "back-end"]);
}

#[test]
fn slugify_drops_apostrophes_instead_of_slugging_them() {
    let global = cfg(vec![Collector::Segments]);
    assert_tags(&global, "ada's-drills/x.typ", &["mt", "adas-drills"]);
}

#[test]
fn slugify_off_keeps_raw_names() {
    let global = DeriveConfig {
        enabled: true,
        collect: vec![Collector::Segments],
        slugify: false,
        ..DeriveConfig::default()
    };
    assert_tags(&global, "Back End/x.typ", &["mt", "Back End"]);
}

#[test]
fn per_root_glob_gates_files() {
    let global = DeriveConfig::default(); // enabled = false globally
    let spec = RootSpec::Full(RootConfig {
        path: "tasks".to_owned(),
        name: Some("mt".to_owned()),
        derive_tags: Some(crate::config::DeriveTags::Glob("backend/**".to_owned())),
    });
    let deriver = RootDeriver::new(&spec, &global);
    assert_eq!(
        deriver.tags_for(Path::new("backend/auth/x.typ")),
        vec!["mt".to_owned(), "backend".to_owned(), "auth".to_owned()]
    );
    assert!(deriver.tags_for(Path::new("frontend/x.typ")).is_empty());
}

#[test]
fn per_root_false_overrides_global_enable() {
    let global = cfg(vec![Collector::Segments]); // enabled = true
    let spec = RootSpec::Full(RootConfig {
        path: "tasks".to_owned(),
        name: Some("mt".to_owned()),
        derive_tags: Some(crate::config::DeriveTags::Enabled(false)),
    });
    let deriver = RootDeriver::new(&spec, &global);
    assert!(deriver.tags_for(Path::new("backend/x.typ")).is_empty());
}

#[test]
fn slugify_avoids_leading_dash() {
    let global = cfg(vec![Collector::Segments]);
    assert_tags(&global, "@hello/x.typ", &["mt", "hello"]);
}

#[test]
fn validate_root_globs_reports_invalid_single_glob() {
    let config = Config {
        tasks: TasksConfig {
            roots: vec![RootSpec::Full(RootConfig {
                path: "tasks".to_owned(),
                name: None,
                derive_tags: Some(DeriveTags::Glob("[invalid".to_owned())),
            })],
            ..TasksConfig::default()
        },
        ..Config::default()
    };
    let problems = validate_root_globs(&config);
    assert!(
        !problems.is_empty(),
        "expected invalid single glob to be reported"
    );
}

#[test]
fn validate_root_globs_reports_invalid_globs_list() {
    let config = Config {
        tasks: TasksConfig {
            roots: vec![RootSpec::Full(RootConfig {
                path: "tasks".to_owned(),
                name: None,
                derive_tags: Some(DeriveTags::Globs(vec![
                    "valid".to_owned(),
                    "[invalid".to_owned(),
                ])),
            })],
            ..TasksConfig::default()
        },
        ..Config::default()
    };
    let problems = validate_root_globs(&config);
    assert!(
        !problems.is_empty(),
        "expected invalid glob in Globs list to be reported"
    );
}
