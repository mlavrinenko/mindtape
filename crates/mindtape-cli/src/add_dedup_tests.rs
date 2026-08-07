#![allow(clippy::unwrap_used)]

use mindtape::naming::DedupMode;

use super::{Collision, DedupArg, effective_mode, resolve};

#[test]
fn dedup_arg_converts_to_naming_dedup_mode() {
    assert_eq!(DedupMode::from(DedupArg::Fail), DedupMode::Fail);
    assert_eq!(DedupMode::from(DedupArg::Skip), DedupMode::Skip);
    assert_eq!(DedupMode::from(DedupArg::AddSuffix), DedupMode::AddSuffix);
}

#[test]
fn effective_mode_flag_wins_over_config() {
    let mut config = mindtape::Config::default();
    config.tasks.naming.dedup = DedupMode::Skip;
    assert_eq!(
        effective_mode(Some(DedupArg::AddSuffix), &config),
        DedupMode::AddSuffix
    );
}

#[test]
fn effective_mode_falls_back_to_config_default() {
    let mut config = mindtape::Config::default();
    config.tasks.naming.dedup = DedupMode::Skip;
    assert_eq!(effective_mode(None, &config), DedupMode::Skip);
}

#[test]
fn effective_mode_falls_back_to_fail_when_nothing_set() {
    let config = mindtape::Config::default();
    assert_eq!(effective_mode(None, &config), DedupMode::Fail);
}

#[test]
fn resolve_reports_no_collision_when_stem_is_free() {
    let out = resolve(DedupMode::Fail, "foo", "tasks/foo.typ", |_| false).unwrap();
    assert_eq!(out, Collision::None);
}

#[test]
fn resolve_fail_errors_naming_the_stem_and_label() {
    let err = resolve(DedupMode::Fail, "foo", "tasks/foo.typ", |_| true).unwrap_err();
    assert_eq!(
        err.to_string(),
        "task \"foo\" already exists at tasks/foo.typ"
    );
}

#[test]
fn resolve_skip_reports_skipped_and_writes_nothing() {
    let out = resolve(DedupMode::Skip, "foo", "tasks/foo.typ", |_| true).unwrap();
    assert_eq!(out, Collision::Skipped);
}

#[test]
fn resolve_add_suffix_mints_the_next_free_stem() {
    let out = resolve(DedupMode::AddSuffix, "foo", "tasks/foo.typ", |candidate| {
        candidate == "foo"
    })
    .unwrap();
    assert_eq!(out, Collision::Suffixed("foo-2".to_owned()));
}

/// `add-suffix` against a predicate that reports every candidate as taken
/// must not loop forever and must surface `naming::disambiguate`'s exhaustion
/// as an error — the fs-free `exists` closure makes this a fast unit test
/// rather than a directory of 10,000 real files.
#[test]
fn resolve_add_suffix_exhausts_into_an_error() {
    let err = resolve(DedupMode::AddSuffix, "foo", "tasks/foo.typ", |_| true).unwrap_err();
    assert!(
        err.to_string().contains("exhausted"),
        "expected an exhaustion error, got: {err}"
    );
}
