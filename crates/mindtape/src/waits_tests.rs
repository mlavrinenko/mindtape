#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::test_support::tempdir;
use std::path::Path;

use super::{Answer, Checker, Permit, annotate_with, command_line, stamp, validate_checkers};
use crate::allow::held_back;
use crate::config::{Config, LegendEntry};
use crate::shell::ConfigCommand;
use crate::test_support::bare_task;

fn checker(name: &str, run: &str) -> Checker {
    Checker {
        name: name.to_owned(),
        run: ConfigCommand::Line(run.to_owned()),
    }
}

fn config(checkers: Vec<Checker>) -> Config {
    let mut config = Config::default();
    config.tasks.waits = checkers;
    config
}

fn line(entry: &str) -> Result<String, String> {
    command_line(entry, &config(vec![checker("up", "check {id} {id}")]))
        .map(|command| command.to_string())
        .map_err(|problem| problem.message)
}

/// `args` as an array command.
fn argv(args: &[&str]) -> ConfigCommand {
    ConfigCommand::Argv(args.iter().map(|arg| (*arg).to_owned()).collect())
}

/// The config holding one `up` checker that runs `args` as an array.
fn array_config(args: &[&str]) -> Config {
    config(vec![Checker {
        name: "up".to_owned(),
        run: argv(args),
    }])
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
    assert_eq!(
        Answer::NotAllowed.reason().as_deref(),
        Some("unknown: not allowed")
    );
    assert_eq!(
        Answer::Nested.reason().as_deref(),
        Some("unknown: asked inside a checker")
    );
}

/// The answer `run` gives for one entry, asked from a fresh directory so no
/// other test's memo answers instead.
#[cfg(not(windows))]
fn answer_of(run: &str) -> Answer {
    let dir = tempdir();
    let mut task = bare_task("t");
    task.waits_on = vec!["up:x".to_owned()];
    stamp(
        &mut task,
        &config(vec![checker("up", run)]),
        dir.path(),
        Permit::Run,
    );
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

/// The mark is what stops an `mt` the checker runs from running checkers.
#[cfg(not(windows))]
#[test]
fn a_checker_runs_with_the_nested_mark_set() {
    assert_eq!(
        answer_of(&format!("echo \"${}\"; exit 1", super::NESTED_ENV)),
        Answer::Waiting("1".to_owned())
    );
}

/// Two tasks, `a` and `b`, each waiting on `up:x`.
fn two_waiting_on_up_x() -> Vec<crate::model::Task> {
    let mut tasks = vec![bare_task("a"), bare_task("b")];
    for task in &mut tasks {
        task.waits_on = vec!["up:x".to_owned()];
    }
    tasks
}

#[cfg(not(windows))]
#[test]
fn a_command_line_runs_once_per_root_however_often_it_is_asked() {
    let dir = tempdir();
    let config = config(vec![checker("up", "echo run >> runs; exit 1")]);
    let mut tasks = two_waiting_on_up_x();

    annotate_with(&mut tasks, &config, dir.path(), || Permit::Run);
    annotate_with(&mut tasks, &config, dir.path(), || Permit::Run);

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

    annotate_with(&mut tasks, &config, Path::new("/no/such/root"), || {
        Permit::Run
    });

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

    stamp(
        &mut task,
        &config(vec![checker("up", "exit 0")]),
        Path::new("/no/such/root"),
        Permit::Run,
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

#[test]
fn a_checker_the_user_has_not_allowed_answers_without_running() {
    let dir = tempdir();
    let run = "echo held >> runs";
    let mut task = bare_task("t");
    task.waits_on = vec!["up:x".to_owned(), "nope:x".to_owned()];

    stamp(
        &mut task,
        &config(vec![checker("up", run)]),
        dir.path(),
        Permit::NotAllowed,
    );

    assert_eq!(task.answers["up:x"], Answer::NotAllowed);
    assert_eq!(
        task.answers["nope:x"],
        Answer::Unknown("unknown checker: 'nope'".to_owned())
    );
    assert!(!dir.path().join("runs").exists());
    assert!(held_back().contains(&run.to_owned()));
}

#[test]
fn annotate_asks_about_the_approval_only_when_an_open_task_waits() {
    let config = config(vec![checker("up", "exit 0")]);
    let mut tasks = vec![bare_task("free")];
    let asked = std::cell::Cell::new(false);

    annotate_with(&mut tasks, &config, Path::new("/no/such/root"), || {
        asked.set(true);
        Permit::NotAllowed
    });

    assert!(!asked.get());
    assert!(tasks[0].answers.is_empty());
}

#[test]
fn annotate_holds_every_open_task_back_when_not_allowed() {
    let config = config(vec![checker("up", "exit 0")]);
    let mut tasks = two_waiting_on_up_x();

    annotate_with(&mut tasks, &config, Path::new("/no/such/root"), || {
        Permit::NotAllowed
    });

    assert_eq!(tasks[0].answers["up:x"], Answer::NotAllowed);
    assert_eq!(tasks[1].answers["up:x"], Answer::NotAllowed);
}

/// An array's `{id}` fills each item alone, so the id stays one argument.
#[test]
fn an_array_checker_fills_the_id_into_each_item() {
    let command = command_line("up:pr-7", &array_config(&["gh", "{id}", "x{id}"]));

    assert_eq!(command.ok(), Some(argv(&["gh", "pr-7", "xpr-7"])));
}

/// An array with no program is as empty as a blank line.
#[test]
fn an_array_with_no_program_is_an_empty_command() {
    assert_eq!(
        validate_checkers(&array_config(&[])),
        ["tasks.waits[0].run: empty command"]
    );
}

/// Inside a checker no checker runs, so one that runs `mt` on its own project
/// cannot spawn itself again.
#[test]
fn inside_a_checker_no_checker_runs() {
    let dir = tempdir();
    let mut tasks = vec![bare_task("t")];
    tasks[0].waits_on = vec!["up:x".to_owned(), "nope:x".to_owned()];

    annotate_with(
        &mut tasks,
        &config(vec![checker("up", "echo ran >> runs")]),
        dir.path(),
        || Permit::Nested,
    );

    assert_eq!(tasks[0].answers["up:x"], Answer::Nested);
    assert_eq!(
        tasks[0].answers["nope:x"],
        Answer::Unknown("unknown checker: 'nope'".to_owned())
    );
    assert!(!dir.path().join("runs").exists());
}
