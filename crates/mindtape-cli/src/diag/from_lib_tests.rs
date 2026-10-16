//! Library errors as `Diag`s: a `ConditionError`, a `typst_cleanup::Error`, and the
//! config, filter, recurrence, reference and breach errors.

use mindtape::config::ConfigError;
use mindtape::recur::dates::RecurError;
use mindtape::{
    ConditionError, DateOverrideError, FilterError, Location, RefError, Severity, Unknown,
    Violation, ViolationKind,
};

use super::render::render;
use super::{Diag, Kind, filter, line};

fn plain(diag: &Diag) -> String {
    render(diag, false)
}

#[test]
fn a_condition_error_is_a_failure_with_a_next_step() {
    let task_field: Diag = ConditionError::TaskField("priority".to_owned()).into();
    assert_eq!(task_field.kind(), Kind::Failed);
    assert_eq!(
        plain(&task_field),
        "error: a condition can't read a task field: 'priority'\n\
         hint: use on_branch(), tty(), env(), weekday(), now() or format()\n"
    );
    let invalid: Diag = ConditionError::Invalid("tty() takes no arguments".to_owned()).into();
    assert_eq!(
        plain(&invalid),
        "error: invalid condition: tty() takes no arguments\n"
    );
}

#[test]
fn every_condition_error_is_a_failure_in_one_shape() {
    let cases = [
        (
            ConditionError::Parse("a '(' is never closed".to_owned()),
            "error: can't parse condition: a '(' is never closed\n",
        ),
        (
            ConditionError::Unsupported("'+' isn't an operator".to_owned()),
            "error: unsupported in condition: '+' isn't an operator\n",
        ),
        (
            ConditionError::UnknownPredicate("ttyy".to_owned()),
            "error: unknown predicate in condition: 'ttyy'\n\
             hint: use on_branch, tty, env, weekday, now, format, nonempty, contested, named or blocked\n",
        ),
    ];
    for (err, expected) in cases {
        let diag = Diag::from(err);
        assert_eq!(diag.kind(), Kind::Failed);
        assert_eq!(plain(&diag), expected);
    }
}

#[test]
fn a_config_error_lists_its_known_names_on_a_hint() {
    let err = ConfigError::Invalid {
        message: "views.a.columns: unknown column: 'fiel'".to_owned(),
        place: None,
        unknown: Some(Unknown::among("fiel", ["file", "title"])),
    };
    assert_eq!(
        plain(&super::from_lib::config(&err)),
        "error: views.a.columns: unknown column: 'fiel'\n\
         hint: did you mean file?\n\
         hint: use one of: file, title\n"
    );
}

#[test]
fn a_list_alone_is_one_hint() {
    let err = ConfigError::Invalid {
        message: "extends: unknown pack: 'x'".to_owned(),
        place: None,
        unknown: Some(Unknown::among("x", ["a", "b"])),
    };
    assert_eq!(
        plain(&super::from_lib::config(&err)),
        "error: extends: unknown pack: 'x'\nhint: use one of: a, b\n"
    );
}

#[test]
fn an_unknown_dict_lists_the_dicts_on_a_hint() {
    let err = FilterError::UnknownDict {
        name: "verb".to_owned(),
        unknown: Unknown::among("verb", ["verbs", "tags"]),
    };
    assert_eq!(
        plain(&Diag::from(err)),
        "error: unknown dict: 'verb'\nhint: did you mean verbs?\nhint: use one of: verbs, tags\n"
    );
}

#[test]
fn an_unknown_path_set_says_how_to_declare_it() {
    let err = FilterError::UnknownPathSet {
        name: "verb".to_owned(),
        unknown: Unknown::among("verb", ["verbs"]),
    };
    assert_eq!(
        plain(&Diag::from(err)),
        "error: unknown path set: '@verb': declare it as [dicts.verb]\n\
         hint: did you mean verbs?\nhint: use one of: verbs\n"
    );
}

fn cleanup_text(err: typst_cleanup::Error) -> String {
    crate::diag::plain(&Diag::from(err))
}

#[test]
fn a_cleanup_failure_is_worded_by_what_went_wrong() {
    use swhid_mint::git::Error as Mint;
    use typst_cleanup::Error;
    let path = || std::path::PathBuf::from("tasks/a.typ");
    let commit = "hint: commit the file first: the tombstone pins its last-touch commit\n";

    assert_eq!(
        cleanup_text(Error::OutsideRepo(path())),
        "error: task file is outside the git repository: 'tasks/a.typ'\n"
    );
    assert_eq!(
        cleanup_text(Error::Mint(path(), Mint::NotTracked("a".into()))),
        format!("error: task file is not tracked by git: 'tasks/a.typ'\n{commit}")
    );
    assert_eq!(
        cleanup_text(Error::Mint(path(), Mint::Dirty("a".into()))),
        format!("error: task file has uncommitted changes: 'tasks/a.typ'\n{commit}")
    );
    assert_eq!(
        cleanup_text(Error::Mint(path(), Mint::Git("boom".into()))),
        "error: tasks/a.typ: git: boom\n"
    );
    let denied = || std::io::Error::from(std::io::ErrorKind::PermissionDenied);
    assert_eq!(
        cleanup_text(Error::Read(path(), denied())),
        "error: can't read: 'tasks/a.typ': permission denied\n"
    );
    assert_eq!(
        cleanup_text(Error::Write(path(), denied())),
        "error: can't write: 'tasks/a.typ': permission denied\n"
    );
    assert_eq!(
        cleanup_text(Error::Git("no repository".into())),
        "error: git: no repository\n"
    );
    let splice = typst_edit::EditError::NotCharBoundary { offset: 1 };
    assert!(
        cleanup_text(Error::Rewrite(path(), splice))
            .starts_with("error: can't rewrite links: 'tasks/a.typ': ")
    );
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
        plain(&Diag::from(RefError::NoMatch {
            reference: "x".to_owned(),
            nearest: None,
        })),
        "error: no task: 'x'\nhint: run mt ls to see your tasks\n"
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
        "error: unknown field: 'wip' in filter 'status == wip'\n\
         hint: to match text, quote it, as in status == \"wip\"\n"
    );
    assert_eq!(plain(&bare), "error: unknown field: 'wip'\n");
    assert_eq!(git.kind(), Kind::Failed);
    assert_eq!(plain(&git), "error: can't read git: changed(): no repo\n");
}

/// The quoting hint quotes the unknown name only where it is compared with a
/// field, in the user's own expression, and never inside a string.
#[test]
fn an_unknown_name_compared_with_a_field_gets_the_quoted_form() {
    let hint = |expr: &str, name: &str| {
        let diag = filter(FilterError::UnknownField(name.to_owned()), Some(expr));
        plain(&diag).lines().nth(1).map(str::to_owned)
    };
    assert_eq!(
        hint("wip != status && title == \"wip\"", "wip").as_deref(),
        Some("hint: to match text, quote it, as in \"wip\" != status && title == \"wip\"")
    );
    assert_eq!(
        hint("priority>high", "high").as_deref(),
        Some("hint: to match text, quote it, as in priority>\"high\"")
    );
    assert_eq!(
        hint("titel == \"x\"", "titel"),
        None,
        "a string on the other side"
    );
    assert_eq!(hint("has(wipe) && wip", "wip"), None, "no comparison");
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
        candidates: vec![
            ("abc".to_owned(), "abacus".to_owned()),
            ("abd".to_owned(), "alpha".to_owned()),
        ],
    };
    assert_eq!(
        plain(&Diag::from(ambiguous)),
        "error: stem hash names several tasks: 'ab'\n\
         hint: pass a longer hash or the stem\n\
         note: candidate: abc abacus\n\
         note: candidate: abd alpha\n"
    );
    let shared = RefError::SharedStem {
        reference: "gamma".to_owned(),
        paths: vec!["../cli/gamma.typ".into(), "../web/gamma.typ".into()],
    };
    assert_eq!(
        plain(&Diag::from(shared)),
        "error: stem names several tasks: 'gamma'\n\
         hint: pass the path of the one you mean\n\
         note: candidate: ../cli/gamma.typ\n\
         note: candidate: ../web/gamma.typ\n"
    );
    let near = RefError::NoMatch {
        reference: "alpah".to_owned(),
        nearest: Some("alpha".to_owned()),
    };
    assert_eq!(
        plain(&Diag::from(near)),
        "error: no task: 'alpah'\nhint: did you mean alpha?\n"
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

/// An `error` breach of `rules.one-wip` at `tasks/x.typ:2`, with nothing to
/// suggest.
pub(super) fn breach() -> Violation {
    Violation {
        task_id: "x".to_owned(),
        field: "rules.one-wip".to_owned(),
        message: "finish the task in progress".to_owned(),
        location: Location::at("tasks/x.typ", 2),
        severity: Severity::Error,
        kind: ViolationKind::default(),
        reads: None,
        unknown: None,
        dict_miss: None,
    }
}

#[test]
fn a_breach_renders_its_place_rule_and_message() {
    let mut violation = breach();
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
    for (template, place) in [
        (Some("task"), "<new task from template 'task'>:2"),
        (None, "<new task>:2"),
    ] {
        violation.location = Location::Filled {
            line: 2,
            template: template.map(str::to_owned),
        };
        let shown = line(&Diag::from(&violation));
        assert_eq!(
            shown,
            format!("warning: {place}: rules.one-wip: finish the task in progress")
        );
    }
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
                unknown: Unknown::among("wpi", ["todo", "wip"]),
            },
            "error: unknown status: 'wpi' in filter 'x'\nhint: did you mean wip?\nhint: use one of: todo, wip\n",
        ),
        (
            FilterError::UnknownStatus {
                value: "zzz".to_owned(),
                unknown: Unknown::among("zzz", ["todo", "wip"]),
            },
            "error: unknown status: 'zzz' in filter 'x'\nhint: use one of: todo, wip\n",
        ),
    ];
    for (err, expected) in cases {
        let diag = filter(err, expr);
        assert_eq!(diag.kind(), Kind::Usage);
        assert_eq!(plain(&diag), expected);
    }
}
