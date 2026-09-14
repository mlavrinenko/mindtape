//! The dogfood project (this repo's own `.mindtape/config.toml` +
//! `truth/tasks/*.typ`) must pass full task validation, not just the
//! JSON-schema shape check `config_schema_validate.rs` runs — otherwise a
//! required field like `estimate` could be flipped on in config while the
//! actual backlog silently fails only via the git pre-commit hook, never
//! `cargo test`.
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

fn root() -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "..", ".."].iter().collect()
}

/// Loaded with both environment-derived directories dropped: through
/// `load_config` this would merge the global `defaults.toml` of whoever runs
/// the suite, and a `[tasks]` line there could fail the repo's own backlog on
/// their machine alone.
fn config() -> mindtape::Config {
    mindtape::load_config_layered_from(&root(), None, None)
        .unwrap()
        .0
}

#[test]
fn dogfood_truth_tasks_pass_validation() {
    let root = root();
    let config = config();
    let outcome = mindtape::load_project(&root, &config);
    assert!(outcome.errors.is_empty(), "{:#?}", outcome.errors);

    let violations = mindtape::validate(&outcome.tasks, &config);
    assert!(violations.is_empty(), "{violations:#?}");
}

/// `kind` names the shape a task takes. The dogfood config declares the
/// vocabulary, so a misspelled or invented kind fails `mt check` rather than
/// silently falling out of every `kind == "umbrella"` filter (the WIP limits).
#[test]
fn dogfood_config_rejects_an_unknown_kind() {
    let root = root();
    let config = config();
    let outcome = mindtape::load_project(&root, &config);
    let mut item = outcome.tasks.first().unwrap().clone();

    for known in ["task", "umbrella"] {
        item.kind = Some(known.to_owned());
        let violations = mindtape::validate(std::slice::from_ref(&item), &config);
        assert!(
            !violations.iter().any(|v| v.field == "kind"),
            "{known}: {violations:#?}"
        );
    }

    item.kind = Some("epic".to_owned());
    let violations = mindtape::validate(std::slice::from_ref(&item), &config);
    assert!(
        violations.iter().any(|v| v.field == "kind"),
        "an unknown kind must be flagged: {violations:#?}"
    );
}

/// Every shipped template stamps its `kind` into the `task.with(…)` head, so a
/// filed task keeps the shape it was filed as. Matched at the head's own
/// two-space indent, where a `slot(…, kind: "int")` argument never sits.
#[test]
fn dogfood_templates_set_a_known_kind() {
    let config = config();
    let allowed = &config.tasks.fields.get("kind").unwrap().values;
    let dir = root().join(".mindtape/templates");
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let source = std::fs::read_to_string(&path).unwrap();
        let kind = source
            .lines()
            .find_map(|line| line.strip_prefix("  kind: \""))
            .and_then(|rest| rest.strip_suffix("\","));
        assert!(
            kind.is_some_and(|kind| allowed.iter().any(|value| value == kind)),
            "{} sets no kind from {allowed:?}: {kind:?}",
            path.display()
        );
    }
}
