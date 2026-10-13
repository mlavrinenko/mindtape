//! End-to-end tests for the completions the binary answers itself
//! (`crates/mindtape-cli/src/cli/completion/mod.rs`).
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
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::tempdir;
use std::path::{Path, PathBuf};

use crate::support;

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
/// minimal's and `proposed` is api-service's, so a baked list, a list read from
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
        "api-service's legend must not leak in: {candidates:?}"
    );
}

/// The other half of the same claim, from the other project: the same argument
/// of the same binary answers differently because the config does.
#[test]
fn flip_offers_a_different_legend_in_a_different_project() {
    let candidates = complete(&example("api-service"), &["mt", "flip", ""]);
    assert!(
        candidates.contains(&"proposed".to_owned()),
        "api-service's legend: {candidates:?}"
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
    let candidates = complete(&example("api-service"), &["mt", "ls", "@"]);
    assert!(
        candidates.iter().any(|c| c.starts_with('@')),
        "view refs: {candidates:?}"
    );
}

/// `mt add -t <TAB>` offers the template stems this project ships, plus the
/// built-in `task` fallback.
#[test]
fn add_offers_template_stems() {
    let candidates = complete(&example("api-service"), &["mt", "add", "-t", ""]);
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

/// A status shortcut verb completes its task argument exactly as the canonical
/// spelling does: `mt done <TAB>` and `mt flip done <TAB>` offer the same list,
/// and that list holds this project's stems.
#[test]
fn a_shortcut_verb_offers_what_flip_offers() {
    let dir = example("minimal");
    let shortcut = complete(&dir, &["mt", "done", ""]);
    let canonical = complete(&dir, &["mt", "flip", "done", ""]);
    assert!(
        shortcut.contains(&"add-dark-mode".to_owned()),
        "minimal's stems: {shortcut:?}"
    );
    assert_eq!(shortcut, canonical);
}

/// The verbs themselves are still offered once each, from the legend.
#[test]
fn the_shortcut_verbs_are_offered_once() {
    let candidates = complete(&example("minimal"), &["mt", "do"]);
    assert_eq!(
        candidates.iter().filter(|c| c.as_str() == "done").count(),
        1,
        "verbs: {candidates:?}"
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
    let dir = tempdir();
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
    let dir = tempdir();
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

/// `mt init --preset <TAB>` offers the preset registry: a built-in's name
/// comes from `mt preset ls`, and an empty user directory still yields them —
/// the one source that loads no project.
#[test]
fn init_offers_the_preset_registry() {
    let candidates = complete(&example("minimal"), &["mt", "init", "--preset", ""]);
    assert!(
        project_values(&candidates)
            .iter()
            .any(|c| c.as_str() == "todone"),
        "built-in presets: {candidates:?}"
    );
}

/// `mt set <task> <TAB>` offers the fields the command writes, each as a
/// whole `field=` token — the same `WRITABLE` table the refusal reads, so a
/// tab press can never offer a field `mt set` would reject.
#[test]
fn set_offers_its_writable_fields() {
    let candidates = complete(&example("minimal"), &["mt", "set", "add-dark-mode", "tit"]);
    assert!(
        candidates.contains(&"title=".to_owned()),
        "writable fields: {candidates:?}"
    );
}

/// Once the token carries a value there is no field name left to complete, so
/// the completer yields nothing rather than guessing.
#[test]
fn set_offers_nothing_once_an_assignment_carries_a_value() {
    let candidates = complete(
        &example("minimal"),
        &["mt", "set", "add-dark-mode", "priority=3"],
    );
    assert!(project_values(&candidates).is_empty(), "{candidates:?}");
}

/// `mt set <task> tags=<TAB>` offers the tags a write is VALIDATED against:
/// api-service declares an `allowed` list, and each candidate is a whole
/// `tags=<tag>` token rather than a bare tag the token grammar would reject.
#[test]
fn set_offers_the_allowed_tags_as_whole_tokens() {
    let candidates = complete(
        &example("api-service"),
        &["mt", "set", "add-redis-client", "tags="],
    );
    assert!(
        candidates.contains(&"tags=security".to_owned()),
        "api-service's tags: {candidates:?}"
    );
    assert!(
        candidates.iter().all(|c| c.starts_with("tags=")),
        "whole tokens only: {candidates:?}"
    );
}

/// With no `[tasks.tags] allowed`, the vocabulary is the tags the tasks
/// actually carry — `mt add --set`'s template-slot grammar completes from the
/// same set, filtered by the prefix typed so far.
#[test]
fn add_set_offers_the_tags_its_tasks_carry() {
    let candidates = complete(
        &example("minimal"),
        &["mt", "add", "-t", "task", "--set", "tags=ba"],
    );
    assert!(
        candidates.contains(&"tags=backend".to_owned()),
        "minimal's tags: {candidates:?}"
    );
}

/// A tag token outside any project resolves its vocabulary to nothing, the
/// quiet failure every source shares — no diagnostic, no candidates.
#[test]
fn a_tag_token_outside_a_project_yields_nothing() {
    let dir = tempdir();
    let candidates = complete(dir.path(), &["mt", "set", "x", "tags="]);
    assert!(project_values(&candidates).is_empty(), "{candidates:?}");
}
