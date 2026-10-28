//! A content rule with its own `message` keeps the check's words too: they
//! print as a `note:` under the message, in `mt check`, in `mt check --json`
//! and in a write the rule refuses.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::test_support::write_file;

use crate::support;

const QUESTIONS: &str = "[tasks]\nroots = [\"tasks\"]\n\n\
     [[tasks.body.content.rules]]\nname = \"no-open-questions\"\npath = [\"TO-BE\"]\n\
     assert = { density = { pattern = '\\?(\\s|$)', max = 0, per = \"sentence\" } }\n\
     when = \"!terminal\"\n";

const ADVICE: &str = "message = \"ask the user and write the answer in\"\n";

/// The breach's place and rule, its task file as `mt` shows it.
fn breach() -> String {
    support::native_in(
        "error: tasks/board.typ:8: body.content.rules.no-open-questions: ",
        &["tasks/board.typ"],
    )
}

const DETAIL: &str = "section `TO-BE` sentence 2 (\"Should it also cover the board?\") \
     must carry at most 0 matches of `\\?(\\s|$)` (found 1)";

/// A project with `config` whose one task, `board`, asks a question in its
/// TO-BE.
fn board(config: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(config);
    write_file(
        dir.path(),
        "tasks/board.typ",
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"Cover the board\",\n  status: accepted(2026, 6, 1),\n)\n\n\
         = TO-BE\n\n- Run `mt ls ?` first.\n- Should it also cover the board?\n",
    );
    dir
}

/// The stderr of `mt <args>` in [`board`], which must exit 1.
fn stderr_in(config: &str, args: &[&str]) -> String {
    let (code, stderr) = support::run_stderr(board(config).path(), args);
    assert_eq!(code, Some(1), "{stderr}");
    stderr
}

#[test]
fn a_custom_message_keeps_the_detail_as_a_note() {
    let stderr = stderr_in(&format!("{QUESTIONS}{ADVICE}"), &["check"]);

    let expected = format!(
        "{}ask the user and write the answer in\nnote: {DETAIL}\n",
        breach()
    );
    assert!(stderr.contains(&expected), "{stderr}");
}

#[test]
fn without_a_message_the_detail_is_the_message_and_no_note_repeats_it() {
    let stderr = stderr_in(QUESTIONS, &["check"]);

    assert!(
        stderr.contains(&format!("{}{DETAIL}\n", breach())),
        "{stderr}"
    );
    assert!(!stderr.contains("note:"), "{stderr}");
}

#[test]
fn json_lists_the_detail_in_notes() {
    let dir = board(&format!("{QUESTIONS}{ADVICE}"));

    let out = support::mt(dir.path())
        .args(["check", "--json"])
        .output()
        .unwrap();
    let (_, stdout, _) = support::parts_of(&out);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let problem = &json["problems"][0];
    assert_eq!(problem["message"], "ask the user and write the answer in");
    assert_eq!(problem["notes"], serde_json::json!([DETAIL]));
}

#[test]
fn a_refused_write_prints_the_detail_too() {
    let stderr = stderr_in(&format!("{QUESTIONS}{ADVICE}"), &["wip", "board"]);

    let expected = format!(
        "{}ask the user and write the answer in\nnote: {DETAIL}\n",
        breach()
    );
    assert!(stderr.contains(&expected), "{stderr}");
}
