//! The `imperative-title` rule the `mindtape-tasks` pack ships, run against
//! titles an engineer files. The rule is loaded through `extends`, so the
//! assertion under test is the one a project inherits, over the dictionaries
//! the binary embeds.
#![allow(clippy::unwrap_used)]

use mindtape::{Filter, Task};

/// Does the pack's `imperative-title` assertion accept `title`?
fn accepted(title: &str) -> bool {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    mindtape::write_atomic(
        &dir.path().join(".mindtape/config.toml"),
        b"extends = [\"mindtape-tasks\"]\n",
    )
    .unwrap();
    let config = mindtape::load_config_layered_from(dir.path(), None, None)
        .unwrap()
        .0;
    let rule = config
        .tasks
        .rules
        .iter()
        .find(|rule| rule.name.as_deref() == Some("imperative-title"))
        .unwrap();
    let task = Task {
        id: "subject".to_owned(),
        title: title.to_owned(),
        ..Task::default()
    };
    Filter::parse(rule.assertion.as_deref().unwrap())
        .unwrap()
        .matches(&task, &config)
        .unwrap()
}

#[test]
fn delete_group_and_a_hyphenated_verb_pass() {
    for title in [
        "Delete the old fixture",
        "Group the index by section",
        "Rate-limit the login endpoint",
    ] {
        assert!(accepted(title), "refused: {title}");
    }
}

#[test]
fn common_hyphenated_verbs_pass() {
    for title in [
        "Re-run the gate after a rebase",
        "Hot-fix the release script",
        "Back-port the fix to the stable branch",
        "Cherry-pick the parser change",
    ] {
        assert!(accepted(title), "refused: {title}");
    }
}

#[test]
fn common_engineering_verbs_pass() {
    let verbs = "update upgrade downgrade disable merge rebase squash retry reset load reload \
                 encode decode audit monitor register schedule initialize reorder restructure \
                 reuse revoke refresh trigger warn default strip quote cap port skip mock batch \
                 stream debug";
    for verb in verbs.split_whitespace() {
        let title = format!("{verb} the thing");
        assert!(accepted(&title), "refused: {title}");
    }
}

#[test]
fn a_title_that_is_not_imperative_is_still_refused() {
    for title in [
        "Loader refactoring",
        "The index grouping",
        "Frobnicate-the loader",
    ] {
        assert!(!accepted(title), "accepted: {title}");
    }
}
