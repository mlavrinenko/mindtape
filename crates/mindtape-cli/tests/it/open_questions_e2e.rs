//! The shipped `mindtape:tasks` pack's `no-open-questions` refuses an open
//! task whose TO-BE, Changes, Out of scope or Dogfood asks a question
//! (`truth/tasks/refuse-a-task-that-leaves-a-question-open.typ`). A `?` in
//! code, or in Problem, is no question.
#![allow(clippy::unwrap_used)]

use mindtape::test_support::write_file;

use crate::support;

const ADVICE: &str = "body.content.rules.no-open-questions: \
     ask the user and write the answer in, since a task leaves no question open";

/// `mt check`'s exit code and stderr for one open task under the pack alone,
/// with `problem` and `to_be` as its Problem and TO-BE bullets.
fn check(problem: &str, to_be: &str) -> (Option<i32>, String) {
    let dir =
        support::scaffold_project("extends = [\"mindtape:tasks\"]\n[tasks]\nroots = [\"tasks\"]\n");
    write_file(
        dir.path(),
        "tasks/board.typ",
        format!(
            "#import \"@local/mindtape:0.2.0\": *\n\n\
             #show: task.with(\n  title: \"Cover the board\",\n  kind: \"refactor\",\n  status: wip(2026, 6, 1),\n)\n\n\
             == Problem\n\n+ Run `mt board`.\n+ 🔴 {problem}\n\n\
             == TO-BE\n\n- 🟢 It lists the tasks.\n{to_be}\n\n\
             == Changes\n\n- `src/board.rs`: list the tasks.\n\n\
             == Out of scope\n\n- The other views.\n"
        ),
    );
    support::run_stderr(dir.path(), &["check"])
}

#[test]
fn a_question_in_to_be_fails_with_the_advice_and_the_sentence() {
    let (code, stderr) = check("It lists nothing.", "- Should it also cover the board?");

    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains(&format!(
            "{ADVICE}\nnote: section `TO-BE` sentence 2 (\"Should it also cover the board?\")"
        )),
        "{stderr}"
    );
}

#[test]
fn a_question_mark_in_code_or_in_problem_passes() {
    let (code, stderr) = check(
        "Why does it list nothing?",
        "- 🟢 `mt board ? --help` names the flag.\n\n```sh\nmt board ? --help\n```",
    );

    assert_eq!(code, Some(0), "{stderr}");
}
