#![allow(clippy::unwrap_used)]

use crate::test_support::tempdir;
use crate::test_support::write_file;

use typst_cst::CommentKind;

use super::{CommentsPolicy, check_comments, shape, validate_comments};
use crate::config::Config;
use crate::test_support::bare_task;
use crate::validate::severity::Severity;

/// Write `body` at `root`/`rel`, creating parent directories as needed.
fn config_with_policy(policy: CommentsPolicy) -> Config {
    let mut config = Config::default();
    config.tasks.comments.policy = policy;
    config
}

#[test]
fn allow_policy_never_scans_even_with_a_comment() {
    let dir = tempdir();
    let task = bare_task("a");
    write_file(dir.path(), &task.file, "// a stray note\n#let x = 1\n");

    let violations = check_comments(&[task], &Config::default(), dir.path()).unwrap();

    assert!(violations.is_empty());
}

#[test]
fn forbid_catches_a_line_comment_with_file_line_and_text() {
    let dir = tempdir();
    let task = bare_task("a");
    write_file(
        dir.path(),
        &task.file,
        "#let x = 1\n// no, use in_view() instead\n",
    );
    let config = config_with_policy(CommentsPolicy::Forbid);

    let violations = check_comments(std::slice::from_ref(&task), &config, dir.path()).unwrap();

    let violation = violations.first().expect("one comment violation");
    assert_eq!(violation.field, "comments");
    assert!(violation.message.contains("no, use in_view() instead"));
    let location = &violation.location;
    assert_eq!(location.file(), Some(task.file.as_path()));
    assert_eq!(location.line(), 2);
}

#[test]
fn forbid_catches_a_block_comment() {
    let dir = tempdir();
    let task = bare_task("a");
    write_file(
        dir.path(),
        &task.file,
        "/* leftover review note */\n#let x = 1\n",
    );
    let config = config_with_policy(CommentsPolicy::Forbid);

    let violations = check_comments(&[task], &config, dir.path()).unwrap();

    let violation = violations.first().expect("one comment violation");
    assert!(violation.message.contains("leftover review note"));
}

#[test]
fn forbid_ignores_double_slash_inside_a_string_literal() {
    let dir = tempdir();
    let task = bare_task("a");
    write_file(
        dir.path(),
        &task.file,
        "#let s = \"http://example.com // not a comment\"\n",
    );
    let config = config_with_policy(CommentsPolicy::Forbid);

    let violations = check_comments(&[task], &config, dir.path()).unwrap();

    assert!(violations.is_empty());
}

#[test]
fn forbid_ignores_double_slash_inside_a_raw_block() {
    let dir = tempdir();
    let task = bare_task("a");
    write_file(
        dir.path(),
        &task.file,
        "```\n// not a comment either\n```\n",
    );
    let config = config_with_policy(CommentsPolicy::Forbid);

    let violations = check_comments(&[task], &config, dir.path()).unwrap();

    assert!(violations.is_empty());
}

#[test]
fn allow_pattern_permits_a_matching_comment_and_still_rejects_others() {
    let dir = tempdir();
    let task = bare_task("a");
    write_file(
        dir.path(),
        &task.file,
        "// TODO: revisit before release\n// no, use in_view() instead\n",
    );
    let mut config = config_with_policy(CommentsPolicy::Forbid);
    config.tasks.comments.allow_pattern = Some("^TODO:".to_owned());

    let violations = check_comments(&[task], &config, dir.path()).unwrap();

    let violation = violations
        .first()
        .expect("the non-matching comment stays a violation");
    assert_eq!(violations.len(), 1);
    assert!(violation.message.contains("no, use in_view() instead"));
}

#[test]
fn a_file_with_no_comments_passes() {
    let dir = tempdir();
    let task = bare_task("a");
    write_file(dir.path(), &task.file, "#let x = 1\n");
    let config = config_with_policy(CommentsPolicy::Forbid);

    let violations = check_comments(&[task], &config, dir.path()).unwrap();

    assert!(violations.is_empty());
}

#[test]
fn warn_policy_still_reports_but_does_not_itself_block() {
    let dir = tempdir();
    let task = bare_task("a");
    write_file(dir.path(), &task.file, "// stray note\n");
    let config = config_with_policy(CommentsPolicy::Warn);

    let violations = check_comments(&[task], &config, dir.path()).unwrap();

    assert_eq!(violations.len(), 1);
    let only = violations.first().unwrap();
    assert_eq!(only.severity, Severity::Warn);
    assert!(!only.is_fatal());
    assert!(!CommentsPolicy::Warn.blocks());
    assert!(CommentsPolicy::Forbid.blocks());
}

#[test]
fn shape_names_each_comment_kind() {
    assert_eq!(shape(&CommentKind::Line), "line");
    assert_eq!(shape(&CommentKind::Block), "block");
}

#[test]
fn validate_comments_rejects_an_uncompilable_allow_pattern() {
    let mut config = Config::default();
    config.tasks.comments.allow_pattern = Some("(".to_owned());

    let problems = validate_comments(&config);

    let problem = problems.first().expect("one problem");
    assert!(problem.contains("allow-pattern"));
}

#[test]
fn validate_comments_accepts_no_pattern() {
    assert!(validate_comments(&Config::default()).is_empty());
}
