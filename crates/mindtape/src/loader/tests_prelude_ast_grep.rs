//! `ast_grep(...)` through the real in-tree prelude: the dict it builds and
//! the scope it refuses.
#![allow(clippy::unwrap_used)]

use typst_harvest::HVal;

use super::tests_prelude::{load_real_prelude, prelude_eval_error};

/// One string field of the probe dict.
fn part<'a>(probe: &'a HVal, key: &str) -> Option<&'a str> {
    probe.get(key).and_then(HVal::as_str)
}

/// The `x-probe` value a task authoring `call` loads with.
fn probe(call: &str) -> HVal {
    let source = format!(
        "#import \"@local/mindtape:{version}\": *\n\
         #show: task.with(title: \"T\", x-probe: {call})\n",
        version = crate::loader::PRELUDE_VERSION
    );
    let (_dir, outcome) = load_real_prelude(&source);
    assert_eq!(outcome.errors, Vec::new(), "`{call}` failed to load");
    outcome
        .tasks
        .first()
        .unwrap()
        .extras
        .get("x-probe")
        .unwrap()
        .clone()
}

/// The hyphenated tag, both strings through the raw-block reader, and the
/// optional trailing glob.
#[test]
fn real_prelude_builds_an_ast_grep_query() {
    let probe = probe("ast_grep(`rust`, `parse_task($$$ARGS)`, \"crates/**\")");
    for (field, value) in [
        ("query", "ast-grep"),
        ("language", "rust"),
        ("pattern", "parse_task($$$ARGS)"),
        ("glob", "crates/**"),
    ] {
        assert_eq!(part(&probe, field), Some(value), "{field}");
    }
}

/// A scope left out reaches the Rust layer as `none`, not a missing key.
#[test]
fn real_prelude_ast_grep_omits_an_unset_glob() {
    let probe = probe("ast_grep(\"rust\", \"fn $NAME()\")");
    assert_eq!(probe.get("glob"), Some(&HVal::Null));
}

/// One optional glob, as `grep(...)` takes one, so a second or a named
/// argument fails the file rather than narrowing nothing.
#[test]
fn real_prelude_refuses_an_ast_grep_with_more_than_one_glob() {
    for call in [
        "ast_grep(\"rust\", \"a\", \"x/**\", \"y/**\")",
        "ast_grep(\"rust\", \"a\", within: \"x/**\")",
    ] {
        let message = prelude_eval_error(&format!("x-probe: {call}"));
        assert!(
            message.contains("ast_grep() takes a language, a pattern and then at most one glob"),
            "{call}: {message}"
        );
    }
}
