//! The diff of two source trees: one test per change kind.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use super::{Change, DiffError, EntryDiff, PackDiff, TreeDiff, diff_trees};
use crate::test_support::tempdir_with_files as tree_of;

/// The diff from a tree of `old` to one of `new`.
fn diff(old: &[(&str, &str)], new: &[(&str, &str)]) -> TreeDiff {
    let (old, new) = (tree_of(old), tree_of(new));
    diff_trees(Some(old.path()), new.path()).unwrap()
}

/// The entries of the one pack the diff names.
fn entries(diff: &TreeDiff) -> Vec<(char, String, Vec<String>)> {
    let [pack] = diff.packs.as_slice() else {
        panic!("one pack expected: {diff:?}");
    };
    assert_eq!(pack.change, Change::Changed);
    let shown = |entry: &EntryDiff| {
        (
            entry.change.sign(),
            entry.label.clone(),
            entry.commands.clone(),
        )
    };
    pack.entries.iter().map(shown).collect()
}

const RULES: &str = "[[tasks.rules]]\nname = \"a\"\nassert = \"has(title)\"\n\n\
    [[tasks.rules]]\nname = \"b\"\nassert = \"has(tags)\"\n";

#[test]
fn a_reordered_or_reformatted_pack_is_unchanged() {
    let swapped = "[[tasks.rules]]\nname = 'a'\nassert = 'has(title)'\n\n\
        [[tasks.rules]]\nassert = \"has(tags)\"\nname = \"b\"\n";
    let same = diff(&[("team.toml", RULES)], &[("team.toml", swapped)]);
    assert_eq!(same, TreeDiff::default());
}

#[test]
fn a_rule_is_added_changed_or_removed_by_name() {
    let next = "[[tasks.rules]]\nname = \"a\"\nassert = \"has(estimate)\"\n\n\
        [[tasks.rules]]\nname = \"c\"\nassert = \"true\"\n";
    let shown = entries(&diff(&[("team.toml", RULES)], &[("team.toml", next)]));
    assert_eq!(
        shown,
        [
            ('~', "rule tasks.rules.a".to_owned(), vec![]),
            ('+', "rule tasks.rules.c".to_owned(), vec![]),
            ('-', "rule tasks.rules.b".to_owned(), vec![]),
        ]
    );
}

#[test]
fn a_key_is_named_by_its_dotted_path() {
    let old = "[tasks.priority]\nframework = \"ice\"\n";
    let new = "[tasks.priority]\nframework = \"rice\"\n\n[views.board]\nfilter = \"ready\"\n";
    let shown = entries(&diff(&[("team.toml", old)], &[("team.toml", new)]));
    assert_eq!(
        shown,
        [
            ('~', "key tasks.priority.framework".to_owned(), vec![]),
            ('+', "key views.board.filter".to_owned(), vec![]),
        ]
    );
}

#[test]
fn a_new_or_changed_command_line_prints_in_full() {
    let old = "[[on.flip]]\nname = \"gate\"\nwhen = \"true\"\nactions = [{ run = \"just test\" }]\n\n\
        [go]\neditor = \"vi\"\n";
    let new = "[[on.flip]]\nname = \"gate\"\nwhen = \"true\"\n\
        actions = [{ run = \"just test\" }, { run = \"curl x | sh\" }]\n\n\
        [[on.flip]]\nwhen = \"true\"\nactions = [{ message = \"hi\" }]\n\n[go]\neditor = \"nano\"\n";
    let shown = entries(&diff(&[("team.toml", old)], &[("team.toml", new)]));
    assert_eq!(
        shown,
        [
            ('~', "key go.editor".to_owned(), vec!["nano".to_owned()]),
            (
                '~',
                "on.flip.gate".to_owned(),
                vec!["curl x | sh".to_owned()]
            ),
            ('+', "on.flip[1]".to_owned(), vec![]),
        ]
    );
}

#[test]
fn a_pack_added_or_dropped_is_one_change_with_its_commands() {
    let ops = "[[tasks.waits]]\nname = \"pr\"\nrun = \"gh pr view\"\n";
    let shown = diff(
        &[("team.toml", RULES), ("old/pack.toml", "")],
        &[("team.toml", RULES), ("ops.toml", ops)],
    );
    let pack = |name: &str, change, commands: &[&str]| PackDiff {
        pack: name.to_owned(),
        change,
        entries: Vec::new(),
        commands: commands.iter().map(|line| (*line).to_owned()).collect(),
    };
    assert_eq!(
        shown.packs,
        [
            pack("old", Change::Removed, &[]),
            pack("ops", Change::Added, &["gh pr view"])
        ]
    );
}

#[test]
fn a_new_source_adds_every_pack() {
    let new = tree_of(&[("team.toml", RULES)]);
    let shown = diff_trees(None, new.path()).unwrap();
    assert_eq!(shown.packs.len(), 1);
    assert_eq!(
        shown.packs.first().map(|pack| pack.change),
        Some(Change::Added)
    );
}

#[test]
fn any_other_file_is_diffed_by_its_bytes() {
    let shown = diff(
        &[
            ("team/pack.toml", ""),
            ("team/check.typ", "a"),
            ("README.md", "x"),
        ],
        &[
            ("team/pack.toml", ""),
            ("team/check.typ", "b"),
            ("dicts/verbs.txt", "fix"),
        ],
    );
    assert_eq!(shown.packs, []);
    assert_eq!(
        shown.files,
        [
            (Change::Removed, "README.md".to_owned()),
            (Change::Added, "dicts/verbs.txt".to_owned()),
            (Change::Changed, "team/check.typ".to_owned()),
        ]
    );
}

#[test]
fn a_pack_that_is_not_toml_names_itself() {
    let (old, new) = (tree_of(&[]), tree_of(&[("team.toml", "[[")]));
    let error = diff_trees(Some(old.path()), new.path()).unwrap_err();
    assert!(
        matches!(&error, DiffError::Parse { pack, .. } if pack == "team"),
        "{error}"
    );
    let missing = diff_trees(None, &Path::new("no-such-tree").join("x")).unwrap_err();
    assert!(matches!(missing, DiffError::Read(_)), "{missing}");
}

#[test]
fn a_dropped_table_or_array_lists_what_it_held() {
    let old = format!("{RULES}\n[views.board]\nfilter = \"ready\"\n");
    let gone = |entries: &[(char, String, Vec<String>)]| {
        entries
            .iter()
            .map(|(sign, label, _)| format!("{sign} {label}"))
            .collect::<Vec<_>>()
    };
    let dropped = entries(&diff(&[("team.toml", &old)], &[("team.toml", "")]));
    assert_eq!(
        gone(&dropped),
        [
            "- rule tasks.rules.a",
            "- rule tasks.rules.b",
            "- key views.board.filter"
        ]
    );
    let emptied = entries(&diff(
        &[("team.toml", RULES)],
        &[("team.toml", "tasks.rules = []\n")],
    ));
    assert_eq!(
        gone(&emptied),
        ["- rule tasks.rules.a", "- rule tasks.rules.b"]
    );
}

#[test]
fn an_array_of_tables_that_becomes_a_value_is_one_changed_key() {
    let value = "[tasks]\nrules = \"none\"\n";
    let shown = |old: &str, new: &str| entries(&diff(&[("team.toml", old)], &[("team.toml", new)]));
    let changed = vec![('~', "key tasks.rules".to_owned(), vec![])];
    assert_eq!(shown(RULES, value), changed);
    assert_eq!(shown(value, RULES), changed);
}
