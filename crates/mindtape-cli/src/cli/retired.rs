//! The commands this binary retired, and the commands that answer in their
//! place.
//!
//! A retired name reaches clap as a status verb, because the legend's shortcut
//! verbs are an `external_subcommand`. `main` answers it here before a project
//! opens. The old spelling then meets its replacement, not an unknown command
//! or a project error.

use crate::diag::{Diag, Hint};

/// The usage error for a command this binary retired. `verb` is the spelling
/// the user typed, and the hint lists the commands that answer in its place.
///
/// `mt explain` retired in favor of the resolved config, the templates and the
/// allow status. A reader with the old spelling then meets the replacement, not
/// the status-verb error.
pub(crate) fn hint(verb: &str) -> Option<Diag> {
    match verb {
        "explain" => Some(
            Diag::usage("unrecognized subcommand 'explain'").hint(Hint::new(
                "run `mt config show --resolved` for the config, `mt templates` for the \
                 templates, or `mt allow --status` for the commands",
            )),
        ),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::hint;

    #[test]
    fn a_retired_command_states_its_replacements() {
        let diag = hint("explain").unwrap();
        let text = crate::diag::plain(&diag);
        assert_eq!(
            text,
            "error: unrecognized subcommand 'explain'\n\
             hint: run `mt config show --resolved` for the config, `mt templates` for the \
             templates, or `mt allow --status` for the commands\n"
        );
    }

    #[test]
    fn an_ordinary_verb_is_no_retired_command() {
        assert!(hint("done").is_none());
    }
}
