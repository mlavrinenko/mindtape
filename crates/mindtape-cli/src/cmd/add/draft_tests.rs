//! The draft path's placement and refusal, without evaluating a source.
#![allow(clippy::unwrap_used)]

use super::draft_rel;
use crate::util::Project;

fn project_with_root(root: &str) -> Project {
    let config = mindtape::parse_config(&format!("[tasks]\nroots = [{root:?}]\n")).unwrap();
    Project {
        root: std::path::PathBuf::from("/project"),
        config,
        outcome: mindtape::LoadOutcome::default(),
        hyperlinks: false,
    }
}

#[test]
fn the_draft_path_sits_under_the_first_root() {
    let project = project_with_root("tasks/fixes");
    assert_eq!(
        draft_rel(&project).unwrap(),
        std::path::Path::new("tasks/fixes/__draft__.typ")
    );
}

#[test]
fn a_root_that_escapes_the_project_is_refused_naming_the_config_key() {
    let err = draft_rel(&project_with_root("../elsewhere")).unwrap_err();
    assert!(crate::diag::plain(&err).contains("tasks.roots"), "{err}");
    assert!(draft_rel(&project_with_root("/elsewhere")).is_err());
}
