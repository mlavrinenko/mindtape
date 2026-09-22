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
    let mut installed: Vec<&str> = crate::install::hook_shells().collect();
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

/// `mt init --preset` offers every built-in `mt preset ls` prints, each
/// helped by its description, then the user presets — a shadowing one once.
#[test]
fn presets_offer_the_builtins_and_the_user_presets() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("mine.toml"), "").unwrap();
    std::fs::write(dir.path().join("flow.toml"), "").unwrap();
    let offered = super::presets_in(Some(dir.path()));
    let names: Vec<String> = offered
        .iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect();
    let mut expected: Vec<String> = crate::preset::list_builtins()
        .into_iter()
        .map(|(name, _)| name.to_owned())
        .collect();
    expected.push("mine".to_owned());
    assert_eq!(names, expected);
    assert!(offered.first().unwrap().get_help().is_some());
}

/// A user preset directory that is missing yields the built-ins alone, never
/// an error.
#[test]
fn presets_without_a_user_directory_are_the_builtins() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("absent");
    let builtins = crate::preset::list_builtins().len();
    assert_eq!(super::presets_in(Some(&missing)).len(), builtins);
    assert_eq!(super::presets_in(None).len(), builtins);
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
