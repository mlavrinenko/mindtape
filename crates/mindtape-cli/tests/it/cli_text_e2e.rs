//! The help text `mt _cli --json` reflects serves a reader whose project has
//! no Justfile. It states only commands `mt` provides, never a recipe from
//! this repo's own Justfile.
//!
//! `just lint::no-just-in-output` scans the binary's own string literals for a
//! recipe mention. It cannot see a `///` doc comment, which is a comment to
//! that scan and the source of clap's help. This test reads the help back out
//! of the parser instead, so it covers a doc comment that becomes `--help`
//! too.
//!
//! The assertion is the word `just ` at all, not one recipe name. Help is a
//! handful of sentences under review, so the simpler, stronger rule is cheap.
//! A `Diag` string is not. The scan reads the recipe list instead.
#![allow(clippy::unwrap_used)]

use crate::support;

/// The text at `key` in `node`, or `?` when it carries none.
fn text<'a>(node: &'a serde_json::Value, key: &str) -> &'a str {
    node.get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("?")
}

/// Add the text at `key` in `node` to `out`, labelled with `at`.
fn push_text(node: &serde_json::Value, at: &str, key: &str, out: &mut Vec<(String, String)>) {
    if let Some(text) = node.get(key).and_then(serde_json::Value::as_str) {
        out.push((at.to_owned(), text.to_owned()));
    }
}

/// Every user-facing help string in the reflected tree, each with the node it
/// sits at (`mt fix`, `mt fix --all`). A failure states the command to fix.
fn help_strings(node: &serde_json::Value, at: &str, out: &mut Vec<(String, String)>) {
    for key in ["about", "long_about", "after_help"] {
        push_text(node, at, key, out);
    }
    let args = node.get("args").and_then(serde_json::Value::as_array);
    for arg in args.into_iter().flatten() {
        let arg_at = format!("{at} {}", text(arg, "id"));
        for key in ["about", "long_about"] {
            push_text(arg, &arg_at, key, out);
        }
    }
    let subs = node
        .get("subcommands")
        .and_then(serde_json::Value::as_array);
    for sub in subs.into_iter().flatten() {
        help_strings(sub, format!("{at} {}", text(sub, "name")).trim(), out);
    }
}

#[test]
fn no_help_text_names_a_just_recipe() {
    let tree = support::cli_tree();

    let mut help = Vec::new();
    help_strings(&tree, "mt", &mut help);
    assert!(
        !help.is_empty(),
        "the dump carried no help text, so this test proves nothing"
    );

    let offenders: Vec<String> = help
        .iter()
        .filter(|(_, text)| text.contains("just "))
        .map(|(at, text)| format!("{at}: {text}"))
        .collect();
    assert!(
        offenders.is_empty(),
        "help says `just `, a recipe only this repo has:\n{}",
        offenders.join("\n")
    );
}
