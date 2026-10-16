#![allow(clippy::unwrap_used)]

use clap::CommandFactory;
use clap_complete::env::Shells;

use super::{Source, decorate, lookup};
use crate::cli::Cli;

/// Every value-taking argument clap declares must be named by `SOURCES`.
///
/// The `just check-doc-facts` arm asserts the same thing through the reflected
/// JSON, which is what makes it visible in CI output; this one fails first and
/// locally, on the commit that adds the argument, without a docs regenerate.
#[test]
fn every_value_taking_argument_names_a_source() {
    let mut missing = Vec::new();
    walk(&Cli::command(), "", &mut |path, arg| {
        let id = arg.get_id().as_str();
        let takes_value = !matches!(
            arg.get_action(),
            clap::ArgAction::SetTrue | clap::ArgAction::SetFalse | clap::ArgAction::Count
        );
        let enumerated = !arg.get_possible_values().is_empty();
        if takes_value
            && !enumerated
            && !matches!(id, "help" | "version")
            && lookup(path, id).is_none()
        {
            missing.push(format!("{path} {id}"));
        }
    });
    assert!(
        missing.is_empty(),
        "arguments with no value source: {missing:?}"
    );
}

/// The reverse: a `SOURCES` entry naming an argument clap no longer declares
/// is a rule that cannot fire, and reads as coverage that is not there.
#[test]
fn every_source_entry_names_a_real_argument() {
    let mut declared = Vec::new();
    walk(&Cli::command(), "", &mut |path, arg| {
        declared.push((path.to_owned(), arg.get_id().as_str().to_owned()));
    });
    let stale: Vec<&str> = super::SOURCES
        .iter()
        .filter(|(path, id, _)| !declared.iter().any(|(cmd, arg)| cmd == path && arg == id))
        .map(|(_, id, _)| *id)
        .collect();
    assert!(
        stale.is_empty(),
        "entries for arguments clap does not declare: {stale:?}"
    );
}

/// `mt install` writes one hook per shell `clap_complete` can answer for. A
/// shell the library gains and this table lacks would install nothing for the
/// users on it, silently.
#[test]
fn the_installed_hooks_cover_every_shell_the_engine_answers_for() {
    let mut engine: Vec<&str> = Shells::builtins().names().collect();
    let mut installed: Vec<&str> = crate::cmd::install::hook_shells().collect();
    engine.sort_unstable();
    installed.sort_unstable();
    assert_eq!(installed, engine);
}

/// Decoration must survive `build()`: an unknown argument id in `SOURCES`
/// panics inside `mut_arg`, and a malformed tree fails its own assertions.
#[test]
fn the_decorated_tree_builds() {
    let mut cmd = decorate(Cli::command());
    cmd.build();
    assert!(cmd.get_subcommands().any(|sub| sub.get_name() == "flip"));
}

/// Every label is distinct: `mt _cli --json` reports it as the name of the
/// source, and two sources sharing one would make the reflection ambiguous.
#[test]
fn source_labels_are_distinct() {
    let labels: Vec<&str> = super::SOURCES
        .iter()
        .filter_map(|(_, _, source)| source.label())
        .collect();
    let mut unique = labels.clone();
    unique.sort_unstable();
    unique.dedup();
    let kinds: std::collections::BTreeSet<&str> = labels.into_iter().collect();
    assert_eq!(kinds.len(), unique.len());
}

/// A `Source::None` entry is a decision, so it has to carry one: an empty
/// reason is the silent skip the table exists to prevent.
#[test]
fn every_declined_argument_gives_a_reason() {
    for (path, id, source) in super::SOURCES {
        if let Source::None(reason) = source {
            assert!(
                !reason.trim().is_empty(),
                "{path} {id} declines with no reason"
            );
        }
    }
}

/// Walk every argument of every command, with its flattened path.
fn walk(cmd: &clap::Command, path: &str, visit: &mut impl FnMut(&str, &clap::Arg)) {
    for arg in cmd.get_arguments() {
        // A global is reflected once, at the root that declares it.
        if path.is_empty() || !arg.is_global_set() {
            visit(path, arg);
        }
    }
    for sub in cmd.get_subcommands() {
        let child = if path.is_empty() {
            sub.get_name().to_owned()
        } else {
            format!("{path} {}", sub.get_name())
        };
        walk(sub, &child, visit);
    }
}

/// `mt _cli --json` reports each source by this name, so the exact spelling is
/// machine-visible. Pins every name and `Source::None`'s `None`; a stale or
/// placeholder return leaves the reflection wrong for every argument.
#[test]
fn source_labels_are_the_reflected_names() {
    assert_eq!(Source::Stems.label(), Some("stems"));
    assert_eq!(Source::Statuses.label(), Some("statuses"));
    assert_eq!(Source::Views.label(), Some("views"));
    assert_eq!(Source::ViewRefs.label(), Some("view-refs"));
    assert_eq!(Source::Dicts.label(), Some("dicts"));
    assert_eq!(Source::Templates.label(), Some("templates"));
    assert_eq!(Source::LinkKinds.label(), Some("link-kinds"));
    assert_eq!(Source::Presets.label(), Some("presets"));
    assert_eq!(Source::Assignment.label(), Some("assignment"));
    assert_eq!(Source::TagToken.label(), Some("tag-token"));
    assert_eq!(Source::FilePath.label(), Some("file-path"));
    assert_eq!(Source::DirPath.label(), Some("dir-path"));
    assert_eq!(Source::None("no project value fits").label(), None);
}

/// Env marker gating the inner side of the `requested` tests below. The
/// predicate reads `$COMPLETE`, so each case re-execs the test binary with the
/// variable staged rather than mutating this process's environment.
const REQUESTED_INNER: &str = "_MT_COMPLETION_REQUESTED";

/// Re-run the calling test in a child with `COMPLETE` set to `complete`
/// (removed for `None`).
fn run_requested_inner(complete: Option<&str>) {
    let out = mindtape::test_support::rerun_isolated_with(REQUESTED_INNER, |child| {
        match complete {
            Some(value) => child.env("COMPLETE", value),
            None => child.env_remove("COMPLETE"),
        };
    });
    assert!(out.is_some(), "only the parent runs the child");
}

/// An ordinary run carries no `$COMPLETE`, so [`super::complete`] must return
/// and let `main` carry on. Pins `requested` against a constant `true`.
#[test]
fn requested_is_false_without_the_complete_variable() {
    if std::env::var(REQUESTED_INNER).is_ok() {
        assert!(!super::requested());
        return;
    }
    run_requested_inner(None);
}

/// A real shell name is a completion run. Pins the positive direction of the
/// predicate.
#[test]
fn requested_is_true_for_a_shell_name() {
    if std::env::var(REQUESTED_INNER).is_ok() {
        assert!(super::requested());
        return;
    }
    run_requested_inner(Some("bash"));
}

/// `$COMPLETE=0` (and the empty string) is explicitly not a completion, so the
/// guard's `name != "0"` must exclude it. Pins the `&&` against `||`: the
/// disjunction reads `0` as a request.
#[test]
fn requested_is_false_for_the_disabled_spellings() {
    if std::env::var(REQUESTED_INNER).is_ok() {
        assert!(!super::requested());
        return;
    }
    run_requested_inner(Some("0"));
    run_requested_inner(Some(""));
}
