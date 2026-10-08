use std::path::Path;

use mindtape::config::ConfigError;
use mindtape::recur::dates::RecurError;
use mindtape::{
    DateOverrideError, FilterError, Location, RefError, Severity, Unknown, Violation, ViolationKind,
};

use super::render::{quote, render, shell_word};
use super::{Command, Diag, Hint, Kind, Note, filter, line};

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
fn a_library_error_is_converted_with_its_own_words() {
    let err = DateOverrideError::InvalidNow("garbage".to_owned());
    assert_eq!(
        plain(&Diag::from(err)),
        "error: MINDTAPE_NOW is not a date: 'garbage'\n\
         hint: set it to YYYY-MM-DD or an RFC 3339 timestamp, or unset it\n"
    );
    assert_eq!(
        plain(&Diag::from(RefError::NoMatch("x".to_owned()))),
        "error: no task: 'x'\n"
    );
}

#[test]
fn a_filter_error_is_a_usage_error_naming_the_filter() {
    let parse = filter(
        FilterError::Parse("Unmatched '('.".to_owned()),
        Some("foo("),
    );
    let field = filter(
        FilterError::UnknownField("wip".to_owned()),
        Some("status == wip"),
    );
    let bare = Diag::from(FilterError::UnknownField("wip".to_owned()));
    let git = filter(
        FilterError::Git {
            call: "changed",
            reason: "no repo".to_owned(),
        },
        Some("changed()"),
    );
    assert_eq!(parse.kind(), Kind::Usage);
    assert_eq!(
        plain(&parse),
        "error: can't parse filter: 'foo(': Unmatched '('\n"
    );
    assert_eq!(
        plain(&field),
        "error: unknown field: 'wip' in filter 'status == wip'\n"
    );
    assert_eq!(plain(&bare), "error: unknown field: 'wip'\n");
    assert_eq!(git.kind(), Kind::Failed);
    assert_eq!(plain(&git), "error: can't read git: changed(): no repo\n");
}

fn toml_error(source: &str) -> toml::de::Error {
    toml::from_str::<toml::Value>(source).unwrap_err()
}

#[test]
fn a_config_error_is_one_line_with_its_place() {
    let parse_in = ConfigError::ParseIn {
        origin: "pack.toml".to_owned(),
        error: toml_error("oops =\n"),
    };
    assert_eq!(
        plain(&Diag::from(parse_in)),
        "error: pack.toml:1:7: can't parse config: invalid string, expected `\"`, `'`\n"
    );
    assert_eq!(
        plain(&Diag::from(ConfigError::Parse(toml_error("oops =\n")))),
        "error: can't parse config: line 1, column 7: invalid string, expected `\"`, `'`\n"
    );
    assert_eq!(
        plain(&Diag::from(ConfigError::invalid("ls.sort: bad"))),
        "error: ls.sort: bad\n"
    );
    let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    assert_eq!(
        plain(&Diag::from(ConfigError::Io(io))),
        "error: can't read config: denied\n"
    );
}

#[test]
fn an_invalid_config_names_its_line_and_turns_a_near_match_into_a_hint() {
    let err = ConfigError::Invalid {
        message: "extends: unknown pack: 'x'\nls.sort: bad".to_owned(),
        place: Some(mindtape::config::ConfigPlace {
            file: ".mindtape/config.toml".into(),
            line: 3,
        }),
        unknown: Some(Unknown {
            nearest: Some("y".to_owned()),
            names: vec!["y".to_owned(), "z".to_owned()],
        }),
    };
    assert_eq!(
        plain(&Diag::from(err)),
        "error: .mindtape/config.toml:3: extends: unknown pack: 'x'\n\
         note: ls.sort: bad\nhint: did you mean y?\nhint: use one of: y, z\n"
    );
}

#[test]
fn a_recurrence_error_fails_with_its_own_words() {
    let cases = [
        (
            RecurError::RruleNotImplemented,
            "error: rrule(...) needs mt built with the rrule feature\n",
        ),
        (
            RecurError::DateOverflow,
            "error: date arithmetic overflowed\n",
        ),
        (
            RecurError::NoMatchingWeekday,
            "error: no weekday in the on: set was reachable within a period\n",
        ),
        (
            RecurError::CatchUpExhausted("2020-01-01".to_owned()),
            "error: recurrence did not reach today from: '2020-01-01'\n",
        ),
        (
            RecurError::RruleExpansion("bad rule".to_owned()),
            "error: can't expand rrule: bad rule\n",
        ),
    ];
    for (err, expected) in cases {
        let diag = Diag::from(err);
        assert_eq!(diag.kind(), Kind::Failed);
        assert_eq!(plain(&diag), expected);
    }
}

#[test]
fn a_reference_error_names_the_reference() {
    let ambiguous = RefError::Ambiguous {
        reference: "ab".to_owned(),
        stems: vec!["alpha".to_owned(), "abacus".to_owned()],
    };
    assert_eq!(
        plain(&Diag::from(ambiguous)),
        "error: stem hash names several tasks: 'ab'\n\
         hint: pass a longer hash or the stem\n\
         note: candidate: alpha\n\
         note: candidate: abacus\n"
    );
    assert_eq!(
        plain(&Diag::from(RefError::NoTaskAt("tasks/x.typ".into()))),
        "error: no task at: 'tasks/x.typ'\n"
    );
    assert_eq!(
        plain(&Diag::from(RefError::Empty)),
        "error: empty task reference\n\
         hint: run mt ls to see your tasks\n"
    );
    let path = RefError::Path {
        path: "nope/x.typ".into(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
    };
    assert_eq!(
        plain(&Diag::from(path)),
        "error: can't resolve path: 'nope/x.typ': not found\n"
    );
}

#[test]
fn a_breach_renders_its_place_rule_and_message() {
    let mut violation = Violation {
        task_id: "x".to_owned(),
        field: "rules.one-wip".to_owned(),
        message: "finish the task in progress".to_owned(),
        location: Location::at("tasks/x.typ", 2),
        severity: Severity::Error,
        kind: ViolationKind::default(),
        reads: None,
        unknown: None,
    };
    let refused = Diag::from(&violation);
    assert_eq!(refused.kind(), Kind::Refused);
    assert_eq!(
        line(&refused),
        "error: tasks/x.typ:2: rules.one-wip: finish the task in progress"
    );
    violation.severity = Severity::Warn;
    violation.location = Location::Pack {
        pack: "mindtape-tasks".to_owned(),
        line: 3,
    };
    let warned = Diag::from(&violation);
    assert_eq!(warned.kind(), Kind::Warning);
    assert_eq!(
        line(&warned),
        "warning: pack mindtape-tasks:3: rules.one-wip: finish the task in progress"
    );
}

#[test]
fn the_other_filter_errors_keep_their_reason_and_kind() {
    let expr = Some("x");
    let cases = [
        (
            FilterError::Unsupported("Const".to_owned()),
            "error: unsupported in filter: 'x': Const\n",
        ),
        (
            FilterError::Invalid("bare field".to_owned()),
            "error: invalid filter: 'x': bare field\n",
        ),
        (
            FilterError::UnknownView("v".to_owned()),
            "error: unknown view: 'v' in filter 'x'\n",
        ),
        (
            FilterError::UnknownDict {
                name: "d".to_owned(),
                unknown: Unknown::among("d", ["a"]),
            },
            "error: unknown dict: 'd' in filter 'x'\nhint: use one of: a\n",
        ),
        (
            FilterError::Call {
                problem: "has() takes a field".to_owned(),
                example: "has(due)",
            },
            "error: invalid filter: 'x': has() takes a field\nhint: for example: has(due)\n",
        ),
        (
            FilterError::UnknownFunction {
                name: "hass".to_owned(),
                nearest: Some("has".to_owned()),
            },
            "error: unknown function: 'hass' in filter 'x'\nhint: did you mean has()?\n",
        ),
        (
            FilterError::UnknownStatus {
                value: "wpi".to_owned(),
                nearest: Some("wip".to_owned()),
            },
            "error: unknown status: 'wpi' in filter 'x'\nhint: did you mean wip?\n",
        ),
        (
            FilterError::UnknownStatus {
                value: "zzz".to_owned(),
                nearest: None,
            },
            "error: unknown status: 'zzz' in filter 'x'\nhint: run mt flip --list to see the statuses\n",
        ),
    ];
    for (err, expected) in cases {
        let diag = filter(err, expr);
        assert_eq!(diag.kind(), Kind::Usage);
        assert_eq!(plain(&diag), expected);
    }
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
