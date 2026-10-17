//! Declared read/write classification for every visible `mt` command — the
//! machine-readable source that replaces hand-typing the CQRS split.
//! `cli/tree_json.rs` annotates each reflected leaf with its `class`, so it rides
//! along into `www/generated/cli.json` for free; `docs/src/contributing.typ`
//! renders the "Command Semantics" bullets from that JSON instead of
//! retyping them, and `tests/it/read_no_write_e2e.rs` generates a per-command
//! read test from the same JSON.
//!
//! A command's key is its bare top-level name (`"ls"`), or — for a command
//! that only exists as a leaf under a parent with subcommands — the
//! space-joined path (`"config show"`, `"preset save"`), the same
//! flattening `cli/tree_json.rs` uses when it computes a node's path. `config`
//! and `preset` carry no entry of their own: neither is a leaf itself, only
//! their children are, and `preset`'s two children land in different
//! classes, so classifying the parent could not even be precise.
//!
//! Exhaustiveness — every visible leaf classified, and no stale entry left
//! behind by a removed command — is asserted in `cli/tree_json.rs`'s own tests,
//! the one place in this crate that sees both `Cli::command()`'s reflected
//! tree and this table at compile time.
//!
//! A read command also carries its [`Reach`]: whether it runs on a set that a
//! leading `@name` holds. `main` refuses a set for every command not marked
//! [`Reach::Set`] before it runs ([`refuses_a_set`]), and `cli/tree_json.rs`
//! exports the mark as `runs_on_set`, so the docs list the set commands from
//! this table.
//!
//! Refs: `truth/tasks/gate-the-cqrs-split-against-the-command-tree.typ`,
//! `truth/tasks/mark-the-commands-that-run-on-a-set-in-the-command-registry.typ`.

/// A command's read/write classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Leaves the authored tree byte-identical, on one project or on a set.
    Read(Reach),
    /// Mutates the authored tree.
    Write,
    /// Neither: exempt from the read/write split, carrying why.
    Exempt(&'static str),
}

/// What a read command runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// One project: a held set is refused.
    One,
    /// One project, or every project of a held set.
    Set,
}

/// `(flattened command name, classification)` for every visible leaf `mt
/// _cli --json` reflects. Order follows `cli/mod.rs`'s own `Command` enum, so a
/// diff here tracks a diff there.
// jscpd:ignore-start
// A table of `(command, class)` rows: rows of one shape are the data, not a
// pasted block.
pub const CLASSIFICATION: &[(&str, Class)] = &[
    ("add", Class::Write),
    ("import", Class::Write),
    ("ls", Class::Read(Reach::Set)),
    ("find", Class::Read(Reach::Set)),
    ("graph", Class::Read(Reach::One)),
    ("views", Class::Read(Reach::One)),
    ("dicts", Class::Read(Reach::One)),
    ("at", Class::Read(Reach::One)),
    ("board", Class::Read(Reach::Set)),
    ("check", Class::Read(Reach::Set)),
    ("allow", Class::Write),
    ("config show", Class::Read(Reach::One)),
    ("config schema", Class::Read(Reach::One)),
    ("rules", Class::Read(Reach::One)),
    ("preset save", Class::Write),
    ("preset ls", Class::Read(Reach::One)),
    ("init", Class::Write),
    ("install", Class::Write),
    ("eval", Class::Read(Reach::One)),
    (
        "go",
        Class::Exempt("hands the file to the configured editor; mutates nothing itself"),
    ),
    ("rm", Class::Write),
    ("mv", Class::Write),
    ("set", Class::Write),
    ("review", Class::Write),
    ("flip", Class::Write),
    ("recur", Class::Write),
    ("link", Class::Write),
];
// jscpd:ignore-end

/// Look up a flattened command name's classification.
#[must_use]
pub fn classify(name: &str) -> Option<Class> {
    CLASSIFICATION
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, c)| c)
        .copied()
}

/// Whether `name` is a read command marked [`Reach::Set`].
#[must_use]
pub fn runs_on_set(name: &str) -> bool {
    classify(name) == Some(Class::Read(Reach::Set))
}

/// Whether `argv` names a command that does not run on a set: one not marked
/// [`Reach::Set`], or one with no class at all (a status verb). A held set is
/// refused for such a command before it runs. An argv clap refuses, or one
/// with no command, is `false`: clap words that itself.
#[must_use]
pub fn refuses_a_set(argv: &[std::ffi::OsString]) -> bool {
    use clap::CommandFactory;
    let Ok(matches) = crate::cli::Cli::command().try_get_matches_from(argv) else {
        return false;
    };
    let Some((name, sub)) = matches.subcommand() else {
        return false;
    };
    let path = sub
        .subcommand_name()
        .map_or_else(|| name.to_owned(), |inner| format!("{name} {inner}"));
    !runs_on_set(&path)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn argv(words: &[&str]) -> Vec<std::ffi::OsString> {
        words.iter().map(std::ffi::OsString::from).collect()
    }

    #[test]
    fn writes_exempts_status_verbs_and_one_project_reads_refuse_a_set() {
        for words in [
            &["mt", "init"][..],
            &["mt", "install"],
            &["mt", "flip", "done", "x"],
            &["mt", "go", "x"],
            &["mt", "preset", "save", "n"],
            &["mt", "done", "x"],
            &["mt", "at"],
            &["mt", "config", "schema"],
            &["mt", "preset", "ls"],
            &["mt", "eval", "x"],
        ] {
            assert!(refuses_a_set(&argv(words)), "{words:?}");
        }
    }

    #[test]
    fn set_reads_and_unparsable_lines_do_not_refuse_a_set() {
        for words in [
            &["mt", "ls"][..],
            &["mt", "find", "x"],
            &["mt", "check"],
            &["mt", "board"],
            &["mt"],
            &["mt", "--no-such-flag"],
        ] {
            assert!(!refuses_a_set(&argv(words)), "{words:?}");
        }
    }

    #[test]
    fn only_the_set_reads_run_on_a_set() {
        let marked: Vec<_> = CLASSIFICATION
            .iter()
            .map(|(name, _)| *name)
            .filter(|name| runs_on_set(name))
            .collect();
        assert_eq!(marked, ["ls", "find", "board", "check"]);
        assert!(!runs_on_set("nope"));
    }

    #[test]
    fn classify_finds_a_top_level_read_command() {
        assert_eq!(classify("ls"), Some(Class::Read(Reach::Set)));
    }

    #[test]
    fn classify_finds_a_nested_leaf() {
        assert_eq!(classify("config schema"), Some(Class::Read(Reach::One)));
        assert_eq!(classify("preset save"), Some(Class::Write));
    }

    #[test]
    fn classify_unknown_name_is_none() {
        assert_eq!(classify("nope"), None);
    }

    #[test]
    fn go_is_exempt_with_a_reason() {
        match classify("go") {
            Some(Class::Exempt(reason)) => assert!(!reason.is_empty()),
            other => panic!("expected go to be exempt, got {other:?}"),
        }
    }

    #[test]
    fn no_duplicate_entries() {
        let mut names: Vec<_> = CLASSIFICATION.iter().map(|(n, _)| *n).collect();
        let before = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), before, "duplicate command_class entry");
    }
}
