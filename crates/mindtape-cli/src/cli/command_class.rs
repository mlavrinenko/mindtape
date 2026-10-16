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
//! Refs: `truth/tasks/gate-the-cqrs-split-against-the-command-tree.typ`.

/// A command's read/write classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Leaves the authored tree byte-identical.
    Read,
    /// Mutates the authored tree.
    Write,
    /// Neither: exempt from the read/write split, carrying why.
    Exempt(&'static str),
}

/// `(flattened command name, classification)` for every visible leaf `mt
/// _cli --json` reflects. Order follows `cli/mod.rs`'s own `Command` enum, so a
/// diff here tracks a diff there.
pub const CLASSIFICATION: &[(&str, Class)] = &[
    ("add", Class::Write),
    ("import", Class::Write),
    ("ls", Class::Read),
    ("find", Class::Read),
    ("graph", Class::Read),
    ("views", Class::Read),
    ("dicts", Class::Read),
    ("worktrees", Class::Read),
    ("check", Class::Read),
    ("allow", Class::Write),
    ("config show", Class::Read),
    ("config schema", Class::Read),
    ("rules", Class::Read),
    ("preset save", Class::Write),
    ("preset ls", Class::Read),
    ("init", Class::Write),
    ("install", Class::Write),
    ("eval", Class::Read),
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

/// Look up a flattened command name's classification.
#[must_use]
pub fn classify(name: &str) -> Option<Class> {
    CLASSIFICATION
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, c)| c)
        .copied()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn classify_finds_a_top_level_read_command() {
        assert_eq!(classify("ls"), Some(Class::Read));
    }

    #[test]
    fn classify_finds_a_nested_leaf() {
        assert_eq!(classify("config schema"), Some(Class::Read));
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
