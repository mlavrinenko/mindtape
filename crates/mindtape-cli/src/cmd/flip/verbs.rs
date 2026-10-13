//! The status-verb surface of `mt flip`: the unknown-status and unknown-command
//! errors with their nearest-match suggestion, the warning for a status that
//! shadows a built-in subcommand, and the `--list` printout.

use std::collections::HashSet;

use mindtape::Config;
use mindtape::config::Legend;

use crate::diag::{Command, Diag, Hint};

/// Configured legend values as `&str`, for a nearest-match suggestion shared
/// by the unknown-status and shortcut-verb-miss sites — both reject against
/// this same set
/// (`truth/tasks/adopt-nearest-match-suggestions-at-the-remaining-unknown-name-errors.typ`).
pub(super) fn legend_values(legend: &Legend) -> Vec<&str> {
    legend.entries.iter().map(|e| e.value.as_str()).collect()
}

/// The usage error for a shortcut-verb miss (`mt <typo> <task>`), naming a
/// near legend value when one clears the similarity floor. Only the message
/// changes here: `verb` still dispatches solely on `legend.contains`.
pub(super) fn unknown_command(verb: &str, legend: &Legend) -> Diag {
    let diag = Diag::usage("unknown command").value(verb);
    diag.did_you_mean_or(
        verb,
        &legend_values(legend),
        Hint::new("run mt --help for commands, or mt flip --list for status verbs"),
    )
}

/// The usage error for a status the legend doesn't declare.
pub(super) fn unknown_status(status: &str, legend: &Legend) -> Diag {
    let diag = Diag::usage("unknown status").value(status);
    diag.did_you_mean_or(
        status,
        &legend_values(legend),
        Hint::run(
            Command::mt().word("flip").word("--list"),
            "to see the statuses",
        ),
    )
}

/// Warn when a legend value collides with a built-in subcommand name: the
/// built-in wins, so that value never reaches `flip` as a shortcut verb.
pub(super) fn warn_shadowed_verbs(config: &Config) {
    let builtins: HashSet<String> = crate::cli::builtin_subcommand_names().into_iter().collect();
    for entry in &config.tasks.status.entries {
        if builtins.contains(&entry.value) {
            crate::diag::emit(
                &Diag::warning("a status shadows the subcommand of its name")
                    .value(&entry.value)
                    .hint(Hint::command(
                        "its shortcut verb won't run; use",
                        Command::mt().word("flip").arg(&entry.value),
                    )),
            );
        }
    }
}

/// Print the per-status shortcut verbs (the configured legend) to stdout.
pub(super) fn list_verbs(config: &Config) {
    let legend = &config.tasks.status;
    if legend.is_empty() {
        println!("no status legend configured; per-status shortcut verbs are unavailable");
        println!("(add [[tasks.status]] entries to enable `mt <status> <task>...`)");
        return;
    }
    println!(
        "per-status shortcut verbs (`mt <status> <task>...` == `mt flip <status> <task>...`):"
    );
    for entry in &legend.entries {
        match &entry.glyph {
            Some(glyph) => println!("  {glyph} {}", entry.value),
            None => println!("  {}", entry.value),
        }
    }
}
