use std::path::Path;

use crate::config::{Collector, DeriveConfig, RootConfig, RootSpec};

use super::RootDeriver;

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

#[test]
fn root_name_defaults_to_last_path_segment() {
    let spec = RootSpec::Short("truth/tasks".to_owned());
    assert_eq!(spec.name(), "tasks");
}

#[test]
fn disabled_global_yields_nothing() {
    let global = DeriveConfig::default(); // enabled = false
    let deriver = RootDeriver::new(&root("mt"), &global);
    assert!(deriver.tags_for(Path::new("backend/auth/x.typ")).is_empty());
}

#[test]
fn root_name_always_emitted_even_at_top_level() {
    let global = cfg(vec![Collector::Segments]);
    let deriver = RootDeriver::new(&root("mt"), &global);
    assert_eq!(deriver.tags_for(Path::new("x.typ")), vec!["mt".to_owned()]);
}

#[test]
fn segments_emits_each_folder() {
    let global = cfg(vec![Collector::Segments]);
    let deriver = RootDeriver::new(&root("mt"), &global);
    assert_eq!(
        deriver.tags_for(Path::new("backend/auth/x.typ")),
        vec!["mt".to_owned(), "backend".to_owned(), "auth".to_owned()]
    );
}

#[test]
fn breadcrumbs_joins_with_separator() {
    let global = cfg(vec![Collector::Breadcrumbs]);
    let deriver = RootDeriver::new(&root("mt"), &global);
    assert_eq!(
        deriver.tags_for(Path::new("backend/auth/x.typ")),
        vec![
            "mt".to_owned(),
            "backend".to_owned(),
            "backend:auth".to_owned()
        ]
    );
}

#[test]
fn both_collectors_dedupe() {
    let global = cfg(vec![Collector::Segments, Collector::Breadcrumbs]);
    let deriver = RootDeriver::new(&root("mt"), &global);
    assert_eq!(
        deriver.tags_for(Path::new("backend/auth/x.typ")),
        vec![
            "mt".to_owned(),
            "backend".to_owned(),
            "auth".to_owned(),
            "backend:auth".to_owned(),
        ]
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
    let deriver = RootDeriver::new(&root("mt"), &global);
    assert_eq!(
        deriver.tags_for(Path::new("backend/x.typ")),
        vec!["mt".to_owned(), "mt:backend".to_owned()]
    );
}

#[test]
fn slugify_normalises_folder_names() {
    let global = cfg(vec![Collector::Segments]);
    let deriver = RootDeriver::new(&root("mt"), &global);
    assert_eq!(
        deriver.tags_for(Path::new("Back End/x.typ")),
        vec!["mt".to_owned(), "back-end".to_owned()]
    );
}

#[test]
fn slugify_off_keeps_raw_names() {
    let global = DeriveConfig {
        enabled: true,
        collect: vec![Collector::Segments],
        slugify: false,
        ..DeriveConfig::default()
    };
    let deriver = RootDeriver::new(&root("mt"), &global);
    assert_eq!(
        deriver.tags_for(Path::new("Back End/x.typ")),
        vec!["mt".to_owned(), "Back End".to_owned()]
    );
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
