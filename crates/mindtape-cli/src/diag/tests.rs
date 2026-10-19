use std::path::Path;

use super::render::{quote, render, shell_word};
use super::{Command, Diag, Hint, Kind, Note};

fn plain(diag: &Diag) -> String {
    render(diag, false)
}

#[test]
fn a_value_is_single_quoted_after_the_text() {
    let diag = Diag::failed("no task").value("nonexistent-xyz");
    assert_eq!(plain(&diag), "error: no task: 'nonexistent-xyz'\n");
}

#[test]
fn a_quote_inside_a_value_is_escaped_as_a_shell_reads_it() {
    assert_eq!(quote("it's"), r"'it'\''s'");
    assert_eq!(quote(""), "''");
}

#[test]
fn a_shell_word_is_bare_when_safe_and_quoted_otherwise() {
    assert_eq!(shell_word("add-redis-client"), "add-redis-client");
    assert_eq!(shell_word("tasks/a_b.typ"), "tasks/a_b.typ");
    assert_eq!(shell_word("@view"), "@view");
    assert_eq!(shell_word("a note"), "'a note'");
    assert_eq!(shell_word("$HOME"), "'$HOME'");
    assert_eq!(shell_word(""), "''");
}

#[test]
fn context_follows_the_value() {
    let diag = Diag::usage("unknown field")
        .value("wip")
        .context("in filter", "status == wip");
    assert_eq!(
        plain(&diag),
        "error: unknown field: 'wip' in filter 'status == wip'\n"
    );
}

#[test]
fn a_place_and_a_rule_lead_the_message() {
    let path = Path::new("tasks/x.typ");
    let with_col = Diag::refused("finish the task in progress")
        .at(path, 2, Some(5))
        .rule("rules.one-wip");
    let without = Diag::refused("finish the task in progress").at(path, 2, None);
    assert_eq!(
        plain(&with_col),
        "error: tasks/x.typ:2:5: rules.one-wip: finish the task in progress\n"
    );
    assert_eq!(
        plain(&without),
        "error: tasks/x.typ:2: finish the task in progress\n"
    );
}

#[test]
fn a_cause_chain_joins_into_one_line_without_a_trailing_period() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "gone.");
    let diag = Diag::failed("can't read").value("/x").cause(&io);
    assert_eq!(plain(&diag), "error: can't read: '/x': gone\n");
}

#[test]
fn hints_and_notes_follow_in_order() {
    let diag = Diag::failed("no task")
        .value("a note")
        .note(Note::new("it names").value("alpha"))
        .note(Note::new("candidate").name("abc alpha"))
        .hint(Hint::did_you_mean("alpha"))
        .hint(Hint::command(
            "to add a note, pass it with -m",
            Command::mt()
                .word("done")
                .arg("alpha")
                .word("-m")
                .arg("a note"),
        ));
    assert_eq!(
        plain(&diag),
        "error: no task: 'a note'\n\
         hint: did you mean alpha?\n\
         hint: to add a note, pass it with -m: mt done alpha -m 'a note'\n\
         note: it names: 'alpha'\n\
         note: candidate: abc alpha\n"
    );
}

#[test]
fn a_run_hint_and_a_text_hint_render_as_written() {
    let run = Hint::run(Command::mt().word("ls"), "to see your tasks");
    assert_eq!(run.to_string(), "run mt ls to see your tasks");
    assert_eq!(Hint::new("quote a value").to_string(), "quote a value");
    assert_eq!(Note::new("parked").to_string(), "parked");
}

#[test]
fn a_members_hints_run_its_commands_in_it_and_a_global_one_bare() {
    let diag = Diag::failed("x")
        .hint(Hint::run(
            Command::mt().word("flip").word("--list"),
            "to see the statuses",
        ))
        .hint(Hint::command(
            "install it once per machine",
            Command::mt_global().word("install"),
        ))
        .in_member("blog");
    assert_eq!(
        plain(&diag),
        "error: x\n\
         hint: run mt @blog flip --list to see the statuses\n\
         hint: install it once per machine: mt install\n\
         note: project: blog\n"
    );
}

#[test]
fn a_members_name_is_quoted_for_the_shell_and_a_text_hint_kept() {
    let diag = Diag::failed("x")
        .hint(Hint::command("list them", Command::mt().word("ls")))
        .hint(Hint::new("run mt ls"))
        .in_member("my blog");
    assert_eq!(
        plain(&diag),
        "error: x\n\
         hint: list them: mt '@my blog' ls\n\
         hint: run mt ls\n\
         note: project: my blog\n"
    );
}

#[test]
#[should_panic(expected = "at most two hints")]
fn a_third_hint_is_refused() {
    let _ = Diag::failed("x")
        .hint(Hint::new("a"))
        .hint(Hint::new("b"))
        .hint(Hint::new("c"));
}

#[test]
fn a_batch_counts_what_landed_and_what_failed() {
    assert_eq!(
        plain(&Diag::batch("flipped", 1, 3)),
        "error: flipped 1 of 3 tasks; 2 failed\n"
    );
    assert_eq!(
        plain(&Diag::batch("flipped", 0, 2)),
        "error: flipped 0 of 2 tasks\n"
    );
    assert_eq!(
        plain(&Diag::batch("flipped", 1, 1)),
        "error: flipped 1 of 1 task\n"
    );
    assert_eq!(
        plain(&Diag::batch("flipped", 2, 2)),
        "error: flipped 2 of 2 tasks\n"
    );
}

#[test]
fn a_warning_says_so_and_the_kind_decides_the_exit_code() {
    assert_eq!(plain(&Diag::warning("odd")), "warning: odd\n");
    assert_eq!(Kind::Usage.code(), 2);
    assert_eq!(Kind::Refused.code(), 1);
    assert_eq!(Kind::Failed.code(), 1);
    assert_eq!(Kind::Warning.code(), 1);
    assert_eq!(Diag::usage("x").kind(), Kind::Usage);
}

#[test]
fn colour_goes_on_the_prefix_only() {
    let diag = Diag::failed("no task")
        .value("x")
        .hint(Hint::new("look"))
        .note(Note::new("n"));
    let warning = render(&Diag::warning("w"), true);
    assert_eq!(
        render(&diag, true),
        "\u{1b}[1m\u{1b}[31merror:\u{1b}[0m no task: 'x'\n\
         \u{1b}[1m\u{1b}[36mhint:\u{1b}[0m look\n\
         \u{1b}[1mnote:\u{1b}[0m n\n"
    );
    assert_eq!(warning, "\u{1b}[1m\u{1b}[33mwarning:\u{1b}[0m w\n");
}

#[test]
fn display_is_the_message_line_alone() {
    let diag = Diag::failed("no task").value("x").hint(Hint::new("look"));
    assert_eq!(diag.to_string(), "no task: 'x'");
}

#[test]
fn a_failed_read_names_the_path_and_the_os_reason() {
    let err = std::fs::read("/nonexistent/nope.typ").unwrap_err();
    let diag = Diag::cannot_read(Path::new("nope.typ"), &err);
    assert_eq!(
        plain(&diag),
        "error: can't read: 'nope.typ': no such file or directory\n"
    );
    assert_eq!(diag.kind(), Kind::Failed);
}

#[test]
fn a_failed_write_says_so_and_a_custom_error_keeps_its_words() {
    let denied = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
    assert_eq!(
        plain(&Diag::cannot_write(Path::new("tasks/a.typ"), &denied)),
        "error: can't write: 'tasks/a.typ': permission denied\n"
    );
    let custom = std::io::Error::other("Disk is full");
    assert_eq!(
        plain(&Diag::failed("can't resolve path").io(&custom)),
        "error: can't resolve path: disk is full\n"
    );
}
