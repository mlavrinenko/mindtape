#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{ChangedFiles, CommitRecord, parse_message, staged_files};
use crate::config::parse_config;
use crate::test_support::bare_task;

fn values(text: &str) -> Vec<String> {
    parse_message(text, "Refs")
        .trailers
        .into_iter()
        .map(|t| t.value)
        .collect()
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
    let (record, problems) =
        CommitRecord::build(&message, &ChangedFiles::default(), &tasks, &config);
    assert_eq!(record.refs, ["parser", "parser", "parsr", "old"]);
    let messages: Vec<&str> = problems.iter().map(|p| p.message.as_str()).collect();
    assert_eq!(messages.len(), 3, "{messages:?}");
    assert!(
        messages[0].contains("write the stem: `Refs: parser`"),
        "{messages:?}"
    );
    assert!(messages[1].contains("names no task"), "{messages:?}");
    assert!(
        messages[1].ends_with("; did you mean: parser?"),
        "suggests the near stem, never the whole backlog: {messages:?}"
    );
    assert!(messages[2].contains("`old` names no task"), "{messages:?}");
}

#[test]
fn tasks_only_needs_every_changed_file_under_a_task_root() {
    let config = parse_config("[tasks]\nroots = [\"truth/tasks\", \"./more/\"]\n").unwrap();
    let only = |paths: &[&str], prefix: &str| {
        let changed = ChangedFiles {
            paths: paths.iter().map(|p| (*p).to_owned()).collect(),
            renamed_from: Vec::new(),
            root_prefix: prefix.to_owned(),
        };
        CommitRecord::build(&parse_message("x", "Refs"), &changed, &[], &config)
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
    let changed = ChangedFiles {
        paths: vec!["a.typ".to_owned()],
        renamed_from: Vec::new(),
        root_prefix: String::new(),
    };
    assert!(
        CommitRecord::build(&parse_message("x", "Refs"), &changed, &[], &everywhere)
            .0
            .tasks_only
    );
}

#[test]
fn outside_a_repository_staged_files_says_so() {
    let dir = tempfile::tempdir().unwrap();
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
    let message = parse_message("x\n\nRefs: tasks/parser\nRefs: parser.typ\n", "Refs");
    let (_, problems) = CommitRecord::build(&message, &ChangedFiles::default(), &tasks, &config);
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
        renamed_from: Vec::new(),
        root_prefix: String::new(),
    };
    let message = parse_message("x\n\nRefs: parser\nRefs: lexer\nRefs: parser\n", "Refs");
    let (record, _) = CommitRecord::build(&message, &changed, &tasks, &config);
    assert_eq!(record.tasks, ["parser", "lexer"]);
}
