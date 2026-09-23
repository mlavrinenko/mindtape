//! Tests for `add` value sources' env-var layering: the implicit
//! `MINDTAPE_ADD_<SLOT>` name, a slot's declared `env`, and when each fills a
//! value or defers. The rest of the value-source surface lives in
//! `add_values_tests.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use mindtape::{RawValue, discover_slots};

use super::{BUILTIN_TASK, env_var_name, layer_env};

#[test]
fn env_var_name_uppercases_and_underscores() {
    assert_eq!(env_var_name("title"), "MINDTAPE_ADD_TITLE");
    assert_eq!(env_var_name("depends-on"), "MINDTAPE_ADD_DEPENDS_ON");
}

#[test]
fn layer_env_fills_unset_skips_seeded_and_blank() {
    let specs = discover_slots(BUILTIN_TASK); // the barebones template: one `title` slot
    let lookup = |key: &str| (key == "MINDTAPE_ADD_TITLE").then(|| "From Env".to_owned());

    let mut values = BTreeMap::new();
    layer_env(&mut values, &specs, lookup);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From Env".to_owned()))
    );

    // A value already set by --set/-f is never overwritten by env.
    let mut seeded = BTreeMap::new();
    seeded.insert("title".to_owned(), RawValue::Str("Explicit".to_owned()));
    layer_env(&mut seeded, &specs, lookup);
    assert_eq!(
        seeded.get("title"),
        Some(&RawValue::Str("Explicit".to_owned()))
    );

    // A blank env value sets nothing.
    let mut blank = BTreeMap::new();
    layer_env(&mut blank, &specs, |_| Some("   ".to_owned()));
    assert!(blank.is_empty());
}

#[test]
fn layer_env_reads_declared_env_name() {
    let specs =
        discover_slots(r#"#f(t: slot("title", env: "MINDTAPE_PRINCIPAL", default: "untitled"))"#);
    let mut values = BTreeMap::new();
    layer_env(&mut values, &specs, |key| {
        (key == "MINDTAPE_PRINCIPAL").then(|| "From Declared Env".to_owned())
    });
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("From Declared Env".to_owned()))
    );
}

#[test]
fn layer_env_declared_env_unset_falls_through() {
    let specs =
        discover_slots(r#"#f(t: slot("title", env: "MINDTAPE_PRINCIPAL", default: "untitled"))"#);
    // The declared variable is unset: nothing fills, and the implicit
    // `MINDTAPE_ADD_TITLE` is not consulted — a declared `env` replaces it.
    let mut values = BTreeMap::new();
    layer_env(&mut values, &specs, |key| {
        (key == "MINDTAPE_ADD_TITLE").then(|| "Implicit".to_owned())
    });
    assert!(values.is_empty());
}
