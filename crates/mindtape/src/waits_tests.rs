#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::Path;

use super::{Answer, Checker, annotate, ask, command_line, validate_checkers};
use crate::config::{Config, LegendEntry};
use crate::test_support::bare_task;

fn checker(name: &str, run: &str) -> Checker {
    Checker {
        name: name.to_owned(),
        run: run.to_owned(),
    }
}

fn config(checkers: Vec<Checker>) -> Config {
    let mut config = Config::default();
    config.tasks.waits = checkers;
    config
}

fn line(entry: &str) -> Result<String, String> {
    command_line(entry, &config(vec![checker("up", "check {id} {id}")]))
        .map_err(|problem| problem.message)
}

#[test]
fn the_command_line_is_the_checkers_run_with_every_id_slot_filled() {
    assert_eq!(line("up:pr-7"), Ok("check pr-7 pr-7".to_owned()));
}

#[test]
fn an_id_may_hold_every_listed_punctuation_and_a_colon() {
    assert_eq!(
        line("up:a1._@/#:+-z"),
        Ok("check a1._@/#:+-z a1._@/#:+-z".to_owned())
    );
}

#[test]
fn an_entry_without_a_colon_is_not_an_entry() {
    assert_eq!(line("pr-7"), Err("'pr-7' is not checker:id".to_owned()));
}

#[test]
fn an_undeclared_checker_is_refused_with_the_nearest_name() {
    let problem = command_line("upp:x", &config(vec![checker("up", "x")])).unwrap_err();

    assert_eq!(problem.message, "unknown checker: 'upp'");
    let unknown = problem.unknown.unwrap();
    assert_eq!(unknown.nearest.as_deref(), Some("up"));
    assert_eq!(unknown.names, ["up"]);
}

#[test]
fn an_id_that_could_reach_the_shell_is_refused() {
    for id in [
        "", "-rf", "#c", "a b", "a;b", "a$b", "a`b`", "a'b", "a%b", "a&b", "é",
    ] {
        let refused = line(&format!("up:{id}"));
        assert!(
            refused
                .as_ref()
                .is_err_and(|message| message.starts_with("id ")),
            "{id:?} gave {refused:?}"
        );
    }
}

#[test]
fn a_reason_marks_an_unknown_answer_and_none_for_a_resolved_one() {
    assert_eq!(Answer::Resolved.reason(), None);
    assert_eq!(
        Answer::Waiting("PR open".to_owned()).reason().as_deref(),
        Some("PR open")
    );
    assert_eq!(
        Answer::Unknown("exit 3".to_owned()).reason().as_deref(),
        Some("unknown: exit 3")
    );
}

/// The answer `run` gives for one entry, asked from a fresh directory so no
/// other test's memo answers instead.
#[cfg(not(windows))]
fn answer_of(run: &str) -> Answer {
    let dir = tempfile::tempdir().unwrap();
    let mut task = bare_task("t");
    task.waits_on = vec!["up:x".to_owned()];
    ask(&mut task, &config(vec![checker("up", run)]), dir.path());
    task.answers["up:x"].clone()
}

#[cfg(not(windows))]
#[test]
fn the_exit_code_decides_the_answer_and_the_first_printed_line_its_reason() {
    assert_eq!(
        answer_of("echo; echo '  PR open '; echo more; exit 1"),
        Answer::Waiting("PR open".to_owned())
    );
    assert_eq!(answer_of("exit 1"), Answer::Waiting("waiting".to_owned()));
    assert_eq!(answer_of("echo fine"), Answer::Resolved);
    assert_eq!(answer_of("exit 2"), Answer::Unknown("exit 2".to_owned()));
    assert_eq!(
        answer_of("echo gone >&2; exit 127"),
        Answer::Unknown("gone".to_owned())
    );
    assert_eq!(
        answer_of("kill -9 $$"),
        Answer::Unknown("terminated by signal".to_owned())
    );
}

#[cfg(not(windows))]
#[test]
fn a_command_line_runs_once_per_root_however_often_it_is_asked() {
    let dir = tempfile::tempdir().unwrap();
    let config = config(vec![checker("up", "echo run >> runs; exit 1")]);
    let mut tasks = vec![bare_task("a"), bare_task("b")];
    for task in &mut tasks {
        task.waits_on = vec!["up:x".to_owned()];
    }

    annotate(&mut tasks, &config, dir.path());
    annotate(&mut tasks, &config, dir.path());

    let runs = std::fs::read_to_string(dir.path().join("runs")).unwrap();
    assert_eq!(runs.lines().count(), 1);
    assert_eq!(
        tasks[1].answers["up:x"],
        Answer::Waiting("waiting".to_owned())
    );
}

#[test]
fn a_terminal_task_is_not_asked_and_a_bad_entry_never_runs() {
    let mut config = config(vec![checker("up", "exit 0")]);
    config.tasks.status.entries = vec![LegendEntry {
        value: "done".to_owned(),
        terminal: true,
        ..LegendEntry::default()
    }];
    let mut done = bare_task("done");
    done.status = Some(crate::model::Status {
        name: "done".to_owned(),
        date: None,
        note: None,
    });
    done.waits_on = vec!["up:x".to_owned()];
    let mut open = bare_task("open");
    open.waits_on = vec!["nope:x".to_owned()];
    let mut tasks = vec![done, open];

    annotate(&mut tasks, &config, Path::new("/no/such/root"));

    assert!(tasks[0].answers.is_empty());
    assert_eq!(
        tasks[1].answers["nope:x"],
        Answer::Unknown("unknown checker: 'nope'".to_owned())
    );
}

#[test]
fn a_shell_that_cannot_start_is_an_unknown_answer() {
    let mut task = bare_task("t");
    task.waits_on = vec!["up:x".to_owned()];

    ask(
        &mut task,
        &config(vec![checker("up", "exit 0")]),
        Path::new("/no/such/root"),
    );

    let Answer::Unknown(reason) = &task.answers["up:x"] else {
        panic!("{:?}", task.answers);
    };
    assert!(reason.starts_with("can't run 'exit 0': "), "{reason}");
}

#[test]
fn checker_names_must_be_spellable_unique_and_run_something() {
    let problems = validate_checkers(&config(vec![
        checker("up", "exit 0"),
        checker("up", " "),
        checker("a:b", "x"),
        checker("", "x"),
        checker("ok_name-2", "x"),
    ]));

    assert_eq!(
        problems,
        [
            "tasks.waits[1].name: 'up' is declared twice",
            "tasks.waits[1].run: empty command",
            "tasks.waits[2].name: 'a:b' must be letters, digits, - and _",
            "tasks.waits[3].name: '' must be letters, digits, - and _",
        ]
    );
}
