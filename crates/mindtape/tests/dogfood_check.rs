//! The dogfood project (this repo's own `.mindtape/config.toml` +
//! `truth/tasks/*.typ`) must pass full task validation, not just the
//! JSON-schema shape check `config_schema_validate.rs` runs — otherwise a
//! required field like `estimate` could be flipped on in config while the
//! actual backlog silently fails only via the git pre-commit hook, never
//! `cargo test`.
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

#[test]
fn dogfood_truth_tasks_pass_validation() {
    let root: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", ".."].iter().collect();
    // Loaded with both environment-derived directories dropped: through
    // `load_config` this would merge the global `defaults.toml` of whoever
    // runs the suite, and a `[tasks]` line there could fail the repo's own
    // backlog on their machine alone.
    let (config, _sources) = mindtape::load_config_layered_from(&root, None, None).unwrap();
    let outcome = mindtape::load_project(&root, &config);
    assert!(outcome.errors.is_empty(), "{:#?}", outcome.errors);

    let violations = mindtape::validate(&outcome.tasks, &config);
    assert!(violations.is_empty(), "{violations:#?}");
}
