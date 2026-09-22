//! End-to-end tests for the completions the binary answers itself
//! (`crates/mindtape-cli/src/completion.rs`).
//!
//! These drive the same protocol the installed shell hook does — `COMPLETE=`
//! plus the word list after `--` — rather than a Rust-level call, because the
//! thing under test is exactly that a tab press reaches a LIVE project. A unit
//! test over a source function would pass while the hook handed clap the wrong
//! argv, which is the failure a user would see.
//!
//! The fixtures are the two checked-in example projects, not tempdirs: the
//! whole claim is "this project's values, not a baked list", and only two real
//! projects whose configs disagree can witness it.
//!
//! Refs: `truth/tasks/emit-shell-completions-from-the-binary-that-answers-them.typ`
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

mod support;

/// An example project's root: `examples/<name>/`, resolved from this crate's
/// manifest so the test is independent of the cwd nextest runs it in.
fn example(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(name)
}

/// The candidates `mt` offers for `words`, with the cursor on the last one.
///
/// `words` is the whole command line as the shell knows it, `mt` included —
/// the hook passes `"${COMP_WORDS[@]}"` verbatim after `--`, and
/// `_CLAP_COMPLETE_INDEX` names the word being completed.
fn complete(dir: &Path, words: &[&str]) -> Vec<String> {
    let index = words.len() - 1;
    let mut args = vec!["--"];
    args.extend_from_slice(words);
    let output = support::mt(dir)
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", index.to_string())
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "completion exited {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    // A completion answers on stdout or not at all: its stderr would land on
    // top of the command line the user is still typing.
    assert!(
        output.stderr.is_empty(),
        "completion wrote to stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

/// The red test this whole change was written against: `mt flip <TAB>` in
/// `examples/minimal` offers THAT project's four-value legend. `todo` is
/// minimal's and `proposed` is learn-piano's, so a baked list, a list read from
/// the wrong project, and the right one are three distinguishable answers.
#[test]
fn flip_offers_the_legend_of_the_project_it_runs_in() {
    let candidates = complete(&example("minimal"), &["mt", "flip", ""]);
    assert!(
        candidates.contains(&"todo".to_owned()),
        "minimal's legend: {candidates:?}"
    );
    assert!(
        !candidates.contains(&"proposed".to_owned()),
        "learn-piano's legend must not leak in: {candidates:?}"
    );
}

/// The other half of the same claim, from the other project: the same argument
/// of the same binary answers differently because the config does.
#[test]
fn flip_offers_a_different_legend_in_a_different_project() {
    let candidates = complete(&example("learn-piano"), &["mt", "flip", ""]);
    assert!(
        candidates.contains(&"proposed".to_owned()),
        "learn-piano's legend: {candidates:?}"
    );
    assert!(
        !candidates.contains(&"todo".to_owned()),
        "minimal's legend must not leak in: {candidates:?}"
    );
}

/// Task stems come from the project's files, so the identity a command takes
/// is the identity the completion offers.
#[test]
fn a_task_argument_offers_this_projects_stems() {
    let candidates = complete(&example("minimal"), &["mt", "go", ""]);
    assert!(
        candidates.contains(&"add-dark-mode".to_owned()),
        "minimal's stems: {candidates:?}"
    );
}

/// `mt ls @<TAB>` is the view half of the TO-BE: the `@name` sugar completes to
/// the views this project declares.
#[test]
fn ls_offers_view_references() {
    let candidates = complete(&example("learn-piano"), &["mt", "ls", "@"]);
    assert!(
        candidates.iter().any(|c| c.starts_with('@')),
        "view refs: {candidates:?}"
    );
}

/// `mt add -t <TAB>` offers the template stems this project ships, plus the
/// built-in `task` fallback.
#[test]
fn add_offers_template_stems() {
    let candidates = complete(&example("learn-piano"), &["mt", "add", "-t", ""]);
    assert!(
        candidates.contains(&"task".to_owned()),
        "template stems: {candidates:?}"
    );
}

/// `mt link <task> <TAB>` offers the edge kinds this project accepts: the six
/// built-ins plus its own `[[tasks.links.kinds]]`.
#[test]
fn link_offers_edge_kinds() {
    let candidates = complete(&example("minimal"), &["mt", "link", "write-docs", ""]);
    assert!(
        candidates.contains(&"depends-on".to_owned()),
        "edge kinds: {candidates:?}"
    );
}

/// Whatever a candidate list still carries once no project value can be in it:
/// clap's own flags, which are the parser's half and correct everywhere.
fn project_values(candidates: &[String]) -> Vec<&String> {
    candidates
        .iter()
        .filter(|candidate| !candidate.starts_with('-'))
        .collect()
}

/// The requirement that keeps a broken tab-press from looking like a broken
/// shell: outside any project there is no project value to offer, and saying
/// so is an exit-0 answer with a silent stderr (asserted in `complete`) rather
/// than a diagnostic. The static half survives — `mt flip`'s flags are clap's
/// to know and do not depend on a project loading.
#[test]
fn a_project_that_does_not_load_yields_no_project_values_and_no_error() {
    let dir = tempfile::tempdir().unwrap();
    let candidates = complete(dir.path(), &["mt", "flip", ""]);
    assert!(
        project_values(&candidates).is_empty(),
        "outside a project: {candidates:?}"
    );
}

/// The same, one step worse: a project whose config is unparseable. `mt flip`
/// itself exits non-zero naming the TOML error; the completer must not.
#[test]
fn an_unparseable_config_yields_no_project_values_and_no_error() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(dir.path().join(".mindtape/config.toml"), "[tasks\n").unwrap();
    let candidates = complete(dir.path(), &["mt", "flip", ""]);
    assert!(
        project_values(&candidates).is_empty(),
        "broken config: {candidates:?}"
    );
}

/// The hook itself: `COMPLETE=<shell> mt` with no word list prints the shell
/// code to source, and that code calls the binary back rather than baking any
/// value into itself.
#[test]
fn the_registration_hook_calls_the_binary_back() {
    let output = support::mt(&example("minimal"))
        .env("COMPLETE", "bash")
        .output()
        .unwrap();
    assert!(output.status.success());
    let hook = String::from_utf8_lossy(&output.stdout);
    assert!(hook.contains("complete"), "{hook}");
    assert!(
        hook.contains("COMPLETE=\"bash\"") || hook.contains("COMPLETE=bash"),
        "{hook}"
    );
}
