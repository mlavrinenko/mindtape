//! Root-aware `[tasks] emit-imports` checks: a module resolves against the
//! project root, which a bare `parse_config` never has.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::tests::project_with_config;

/// A module the project lacks fails the typed load, not one command later, and
/// the error specifies the key and the path.
#[test]
fn a_missing_emit_import_fails_the_load() {
    let (_tmp, home, project) =
        project_with_config("[tasks]\nemit-imports = [\"/truth/prelude/2/lib.typ\"]\n");

    let err = super::load_config_layered_from(&project, Some(&home), None).unwrap_err();
    let crate::config::ConfigError::Invalid { message, .. } = err else {
        panic!("a missing emit import must be a semantic config error: {err}");
    };
    assert_eq!(
        message,
        "tasks.emit-imports: `/truth/prelude/2/lib.typ` does not exist under the project \
         root; write the module or drop the entry"
    );
}
