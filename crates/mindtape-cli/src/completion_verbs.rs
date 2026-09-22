//! Status shortcut verbs, grafted onto the tree a completion run parses.
//!
//! An ordinary run catches `mt done <task>` with a clap `external_subcommand`,
//! whose tail clap never parses. A completion run cannot use that shape, so
//! [`super::decorate`] grafts each legend value on as a real subcommand
//! carrying `flip`'s own arguments — `status` aside, the verb being the
//! status — already decorated under `flip`'s [`super::SOURCES`] entries.
//! `mt done <TAB>` therefore offers exactly what `mt flip done <TAB>` does,
//! and a config adding a status gets its verb and its completion together.
//! The dispatch is untouched: only a completion run ever sees the grafts.
//!
//! Refs: `truth/tasks/complete-a-status-shortcut-verbs-task-argument.typ`

use clap::Command;

/// The loaded project's legend values, none when it did not load.
pub(super) fn legend_verbs() -> Vec<String> {
    super::project()
        .map(|project| {
            project
                .config
                .tasks
                .status
                .entries
                .iter()
                .map(|entry| entry.value.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// Add one subcommand per `verbs` entry, each carrying `flip`'s arguments but
/// `status` (the verb is the status), completers already hung. A verb
/// shadowed by a built-in command is left to that command, as the dispatch
/// leaves it.
pub(super) fn graft(mut cmd: Command, verbs: &[String]) -> Command {
    let Some(flip) = cmd.find_subcommand("flip").cloned() else {
        return cmd;
    };
    for verb in verbs {
        if cmd.find_subcommand(verb).is_some() {
            continue;
        }
        let args = flip
            .get_arguments()
            .filter(|arg| !matches!(arg.get_id().as_str(), "status" | "help"))
            .cloned();
        let shortcut = Command::new(verb.clone())
            .about(format!("Shortcut for `mt flip {verb}`"))
            .args(args);
        cmd = cmd.subcommand(shortcut);
    }
    cmd
}
