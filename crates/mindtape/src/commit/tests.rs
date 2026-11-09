#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{ChangedFiles, CommitRecord, parse_message, staged_files};
use crate::config::parse_config;
use crate::test_support::bare_task;
use crate::test_support::tempdir;
#[cfg(unix)]
use crate::test_support::{commit_files, git_as_tester, git_tempdir};

fn values(text: &str) -> Vec<String> {
    parse_message(text, "Refs")
        .trailers
        .into_iter()
        .map(|t| t.value)
        .collect()
}

/// Build a record from `text` with no changed files.
fn build(
    config: &crate::config::Config,
    tasks: &[crate::model::Task],
    text: &str,
) -> (CommitRecord, Vec<super::RefProblem>) {
    let root = tempdir();
    CommitRecord::build(
        &parse_message(text, "Refs"),
        &ChangedFiles::default(),
        tasks,
        config,
        root.path(),
    )
}

/// A `ChangedFiles` for repository-relative `paths` under `prefix`.
fn changed(paths: &[&str], prefix: &str) -> ChangedFiles {
    ChangedFiles {
        paths: paths.iter().map(|path| (*path).to_owned()).collect(),
        symlinks: Vec::new(),
        renamed_from: Vec::new(),
        root_prefix: prefix.to_owned(),
    }
}

#[test]
fn the_subject_is_the_first_uncommented_line() {
    let message = parse_message("# a comment\n\n  feat: parse  \n\nbody\n", "Refs");
    assert_eq!(message.subject, "feat: parse");
    assert_eq!(parse_message("", "Refs").subject, "");
}

#[test]
fn trailers_come_from_the_last_paragraph_only() {
    assert_eq!(
        values("feat: x\n\nbody\n\nRefs: a\nrefs: b\nOther: c\n"),
        ["a", "b"]
    );
    assert!(values("feat: x\n\nRefs: a\n\nplain prose last\n").is_empty());
    assert!(
        values("Refs: a\n").is_empty(),
        "a subject is never a trailer"
    );
    assert!(values("feat: x\n\nRefs: a\nnot a trailer line\n").is_empty());
}

#[test]
fn every_trailer_line_is_kept_as_written_for_the_trailers_field() {
    let message = parse_message(
        "feat: x\n\nbody\n\nChangelog: none\nRefs: a\n  continued\n",
        "Refs",
    );
    assert_eq!(message.trailer_lines, ["Changelog: none", "Refs: a"]);
    assert!(
        parse_message("Changelog: none\n", "Refs")
            .trailer_lines
            .is_empty()
    );
}

#[test]
fn a_record_carries_the_trailer_lines() {
    let root = tempdir();
    let message = parse_message("x\n\nChangelog: none\n", "Refs");
    let (record, _) = CommitRecord::build(
        &message,
        &ChangedFiles::default(),
        &[],
        &crate::config::Config::default(),
        root.path(),
    );
    assert_eq!(record.trailers, ["Changelog: none"]);
}

#[test]
fn a_continuation_line_stays_in_the_block() {
    assert_eq!(
        values("feat: x\n\nRefs: a\n  continued\nRefs: b\n"),
        ["a", "b"]
    );
}

#[test]
fn comments_and_the_scissors_diff_are_ignored() {
    let text = "feat: x\n\nRefs: a\n# Please enter the commit message\n\
                # ------------------------ >8 ------------------------\n\
                diff --git a/x b/x\n\nRefs: b\n";
    let message = parse_message(text, "Refs");
    assert_eq!(message.trailers.len(), 1);
    assert_eq!(message.trailers[0].value, "a");
    assert_eq!(message.trailers[0].line, 3);
}

#[test]
fn a_record_resolves_stems_and_refuses_paths_and_unknown_values() {
    let config = parse_config("").unwrap();
    let tasks = [bare_task("parser")];
    let message = parse_message(
        "feat: x\n\nRefs: parser\nRefs: tasks/parser.typ\nRefs: parsr\nRefs: gone/old.typ\n",
        "Refs",
    );
    let root = tempdir();
    let (record, problems) = CommitRecord::build(
        &message,
        &ChangedFiles::default(),
        &tasks,
        &config,
        root.path(),
    );
    assert_eq!(record.refs, ["parser", "parser", "parsr"]);
    assert_eq!(
        record.tasks,
        ["parser", "parsr"],
        "a path naming no task contributes no stem"
    );
    let messages: Vec<&str> = problems.iter().map(|p| p.message.as_str()).collect();
    assert_eq!(messages.len(), 3, "{messages:?}");
    assert!(
        messages[0].contains("write the stem: `Refs: parser`"),
        "{messages:?}"
    );
    assert!(messages[1].contains("names no task"), "{messages:?}");
    let near = problems[1].unknown.as_ref().unwrap();
    assert_eq!(
        near.nearest.as_deref(),
        Some("parser"),
        "suggests the near stem"
    );
    assert!(near.names.is_empty(), "never the whole backlog: {near:?}");
    assert!(messages[2].contains("`old` names no task"), "{messages:?}");
}

#[test]
fn tasks_only_needs_every_changed_file_under_a_task_root() {
    let config = parse_config("[tasks]\nroots = [\"truth/tasks\", \"./more/\"]\n").unwrap();
    let root = tempdir();
    let only = |paths: &[&str], prefix: &str| {
        CommitRecord::build(
            &parse_message("x", "Refs"),
            &changed(paths, prefix),
            &[],
            &config,
            root.path(),
        )
        .0
        .tasks_only
    };
    assert!(only(&["truth/tasks/a.typ", "more/b/c.typ"], ""));
    assert!(!only(&["truth/tasks/a.typ", "src/lib.rs"], ""));
    assert!(!only(&["truth/tasks-old/a.typ"], ""));
    assert!(!only(&["truth/tasks/notes.md"], ""));
    assert!(!only(&[], ""), "an empty commit changes no task file");
    assert!(only(&["sub/truth/tasks/a.typ"], "sub/"));
    assert!(!only(&["truth/tasks/a.typ"], "sub/"), "outside the project");
    let everywhere = parse_config("[tasks]\nroots = [\".\"]\n").unwrap();
    assert!(
        CommitRecord::build(
            &parse_message("x", "Refs"),
            &changed(&["a.typ"], ""),
            &[],
            &everywhere,
            root.path(),
        )
        .0
        .tasks_only
    );
}

#[test]
fn a_template_or_reserved_path_under_a_root_dot_is_not_a_task_file() {
    let config =
        parse_config("[tasks]\nroots = [\".\"]\n\n[templates]\ndirs = [\"templates\"]\n").unwrap();
    let root = tempdir();
    let record = |paths: &[&str]| {
        CommitRecord::build(
            &parse_message("x", "Refs"),
            &changed(paths, ""),
            &[],
            &config,
            root.path(),
        )
        .0
    };
    let template = record(&["templates/task.typ"]);
    assert!(!template.tasks_only);
    assert!(template.tasks.is_empty());
    assert!(
        !record(&[".mindtape/tasks/a.typ"]).tasks_only,
        "a reserved component is never a task file"
    );
    let task = record(&["tasks/a.typ"]);
    assert!(task.tasks_only);
    assert_eq!(task.tasks, ["a"]);
}

#[test]
fn a_tracked_deleted_path_is_still_a_task_file() {
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\n").unwrap();
    let root = tempdir();
    let (record, problems) = CommitRecord::build(
        &parse_message("x", "Refs"),
        &changed(&["tasks/gone.typ"], ""),
        &[],
        &config,
        root.path(),
    );
    assert!(problems.is_empty(), "{problems:?}");
    assert!(
        record.tasks_only,
        "classification is lexical, not by existence"
    );
    assert_eq!(record.tasks, ["gone"]);
}

#[test]
fn a_symlink_path_is_not_a_task_file() {
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\n").unwrap();
    let root = tempdir();
    let changed = ChangedFiles {
        paths: vec!["tasks/a.typ".to_owned()],
        symlinks: vec!["tasks/a.typ".to_owned()],
        renamed_from: Vec::new(),
        root_prefix: String::new(),
    };

    let (record, _) = CommitRecord::build(
        &parse_message("x", "Refs"),
        &changed,
        &[bare_task("a")],
        &config,
        root.path(),
    );
    assert!(!record.tasks_only, "a symlink is not a task file");
    assert!(record.tasks.is_empty(), "{:?}", record.tasks);
}

#[test]
fn a_root_outside_the_project_is_not_a_commit_task_file() {
    let project = tempdir();
    let sub = project.path().join("sub");
    let config = parse_config("[tasks]\nroots = [\"../shared/tasks\"]\n").unwrap();
    let (record, _) = CommitRecord::build(
        &parse_message("x", "Refs"),
        &changed(&["shared/tasks/a.typ"], "sub/"),
        &[],
        &config,
        &sub,
    );
    assert!(!record.tasks_only);
    assert!(record.tasks.is_empty());
}

#[cfg(unix)]
#[test]
fn staged_files_marks_a_symlink_entry() {
    let dir = git_tempdir();
    commit_files(dir.path(), &[("tasks/a.typ", "")], "seed");
    std::os::unix::fs::symlink("a.typ", dir.path().join("tasks/link.typ")).unwrap();
    git_as_tester(dir.path(), &["add", "tasks/link.typ"]);

    let changed = staged_files(dir.path()).unwrap();
    assert_eq!(changed.paths, ["tasks/link.typ"]);
    assert_eq!(changed.symlinks, ["tasks/link.typ"]);
}

#[test]
fn outside_a_repository_staged_files_says_so() {
    let dir = tempdir();
    let err = staged_files(dir.path()).unwrap_err();
    assert!(err.to_string().contains("not a git repository"), "{err}");
}

#[test]
fn a_colon_line_with_no_key_is_not_a_trailer_line() {
    assert!(
        values("feat: x\n\nRefs: a\n: b\n").is_empty(),
        "a line whose token before `:` is empty is not a trailer"
    );
}

#[test]
fn a_hyphenated_trailer_key_is_a_trailer_line() {
    let message = parse_message("feat: x\n\nTask-Ref: parser\n", "Task-Ref");
    assert_eq!(message.trailers.len(), 1, "{:?}", message.trailers);
    assert_eq!(message.trailers[0].value, "parser");
}

#[test]
fn a_value_that_is_a_path_or_ends_in_typ_is_refused() {
    let config = parse_config("").unwrap();
    let tasks = [bare_task("parser")];
    let root = tempdir();
    let message = parse_message("x\n\nRefs: tasks/parser\nRefs: parser.typ\n", "Refs");
    let (_, problems) = CommitRecord::build(
        &message,
        &ChangedFiles::default(),
        &tasks,
        &config,
        root.path(),
    );
    let messages: Vec<&str> = problems.iter().map(|p| p.message.as_str()).collect();
    assert_eq!(messages.len(), 2, "{messages:?}");
    assert!(
        messages.iter().all(|m| m.contains("is a path")),
        "a path-slash value and a bare `.typ` value are both paths: {messages:?}"
    );
}

#[test]
fn tasks_dedup_keeping_order() {
    let config = parse_config("[tasks]\nroots = [\"truth/tasks\"]\n").unwrap();
    let tasks = [bare_task("parser"), bare_task("lexer")];
    let changed = ChangedFiles {
        paths: vec![
            "truth/tasks/parser.typ".to_owned(),
            "truth/tasks/lexer.typ".to_owned(),
        ],
        symlinks: Vec::new(),
        renamed_from: Vec::new(),
        root_prefix: String::new(),
    };
    let root = tempdir();
    let message = parse_message("x\n\nRefs: parser\nRefs: lexer\nRefs: parser\n", "Refs");
    let (record, _) = CommitRecord::build(&message, &changed, &tasks, &config, root.path());
    assert_eq!(record.tasks, ["parser", "lexer"]);
}

#[test]
fn a_bare_dotted_value_is_the_stem_as_written() {
    assert_eq!(super::value_stem("foo.task-261023"), "foo.task-261023");
    assert_eq!(
        super::value_stem("tasks/foo.task-261023.typ"),
        "foo.task-261023"
    );
    assert_eq!(super::value_stem("foo.task-261023.typ"), "foo.task-261023");
}

#[test]
fn a_dotted_stem_trailer_names_its_task() {
    let config = parse_config("").unwrap();
    let tasks = [bare_task("foo.task-261023")];
    let message = parse_message("x\n\nRefs: foo.task-261023\n", "Refs");
    let root = tempdir();
    let (record, problems) = CommitRecord::build(
        &message,
        &ChangedFiles::default(),
        &tasks,
        &config,
        root.path(),
    );
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(record.refs, ["foo.task-261023"]);
}

#[test]
fn a_ref_is_a_task_then_an_external_ref_then_refused() {
    let tasks = [bare_task("parser")];
    let config =
        parse_config("[commits]\nexternal-refs = [\"^r#[0-9]+$\", \"^[A-Z][A-Z0-9]+-[0-9]+$\"]\n")
            .unwrap();
    let (record, problems) = build(
        &config,
        &tasks,
        "x\n\nRefs: parser\nRefs: r#45782\nRefs: ABC-12\n",
    );
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(record.refs, ["parser", "r#45782", "ABC-12"]);
    assert_eq!(
        record.tasks,
        ["parser"],
        "an external ref runs no task-scope rule"
    );

    for value in ["parser r#45782", "parser, r#45782"] {
        let (record, problems) = build(&config, &tasks, &format!("x\n\nRefs: {value}\n"));
        assert!(problems.is_empty(), "{value:?}: {problems:?}");
        assert_eq!(record.refs, ["parser", "r#45782"], "{value:?}");
        assert_eq!(record.tasks, ["parser"], "{value:?}");
    }

    let wins = parse_config("[commits]\nexternal-refs = [\".*\"]\n").unwrap();
    let (record, problems) = build(&wins, &tasks, "x\n\nRefs: parser\n");
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(record.refs, ["parser"]);
    assert_eq!(record.tasks, ["parser"]);

    let (record, problems) = build(&wins, &tasks, "x\n\nRefs: tasks/parser.typ\n");
    assert_eq!(record.tasks, ["parser"], "a task path is still a task ref");
    assert_eq!(problems.len(), 1, "but the path itself is refused");
    assert!(problems[0].message.contains("is a path"), "{problems:?}");

    let (record, problems) = build(&config, &tasks, "x\n\nRefs: parsr\n");
    assert_eq!(record.refs, ["parsr"]);
    assert_eq!(problems.len(), 1);
    assert!(
        problems[0].message.contains("names no task"),
        "{problems:?}"
    );

    let spaced = parse_config("[commits]\nexternal-refs = [\"^r#[0-9 ]+$\"]\n").unwrap();
    let (record, problems) = build(&spaced, &[], "x\n\nRefs: \"r#45 782\"\n");
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(record.refs, ["r#45 782"]);
}

#[test]
fn an_external_ref_that_looks_like_a_path_is_kept_opaque() {
    let config = parse_config("[commits]\nexternal-refs = [\"^https://\"]\n").unwrap();
    let root = tempdir();
    let message = parse_message("x\n\nRefs: https://tracker.example/issue/7\n", "Refs");
    let (record, problems) = CommitRecord::build(
        &message,
        &ChangedFiles::default(),
        &[],
        &config,
        root.path(),
    );
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(record.refs, ["https://tracker.example/issue/7"]);
    assert!(record.tasks.is_empty());
}

#[test]
fn a_path_to_a_stem_with_a_separator_suggests_a_quoted_stem() {
    let config = parse_config("").unwrap();
    let tasks = [bare_task("write docs")];
    let (_, problems) = build(&config, &tasks, "x\n\nRefs: \"tasks/write docs.typ\"\n");
    assert_eq!(problems.len(), 1);
    assert!(
        problems[0].message.contains("`Refs: \"write docs\"`"),
        "{problems:?}"
    );
}
