//! Tests for the editor session: spawning the configured editor, tty/mode
//! gating (`should_open`, `has_unfilled_required`), and `run`'s parse of the
//! edited buffer. Scaffold rendering and parse-back live in
//! `cmd/add/interactive_tests.rs`; shares its `specs()`/`TEMPLATE` fixture.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::test_support::tempdir;
use std::collections::BTreeMap;

use mindtape::{AddConfig, InteractiveMode, RawValue, discover_slots};

use super::tests::specs;
use super::{editor_command, open_in_editor, run, should_open};

#[test]
fn editor_command_prefers_override() {
    assert_eq!(editor_command(Some("nvim -p")), "nvim -p");
    assert_eq!(editor_command(Some("  ")), fallback_editor());
}

fn fallback_editor() -> String {
    // VISUAL/EDITOR may be set in the test environment; accept any non-empty.
    let resolved = editor_command(None);
    assert!(!resolved.is_empty());
    resolved
}

// ── editor_command: a blank override falls through, it is not accepted ──

#[test]
fn editor_command_env_var_not_empty_used() {
    const CHILD: &str = "_MT_EDITOR_CMD_CHILD";
    if std::env::var(CHILD).is_ok() {
        // Child: report the result of editor_command(None) via stderr so
        // the test harness (which writes to stdout) does not interfere.
        let result = editor_command(None);
        eprintln!("__MT_EDITOR={result}");
        return;
    }
    // Parent: spawn a child with VISUAL set and EDITOR removed so the
    // env-var path is deterministic.
    let exe = std::env::current_exe().unwrap();
    let output = std::process::Command::new(&exe)
        .env(CHILD, "1")
        .env("VISUAL", "custom-editor")
        .env_remove("EDITOR")
        .arg("editor_command_env_var_not_empty_used")
        .arg("--nocapture")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Parse the sentinel line from stderr (the harness writes nothing to it).
    let stderr = String::from_utf8_lossy(&output.stderr);
    let sentinel = stderr
        .lines()
        .find(|l| l.starts_with("__MT_EDITOR="))
        .unwrap_or_else(|| panic!("missing sentinel in stderr:\n{stderr}"));
    let child_result = sentinel.strip_prefix("__MT_EDITOR=").unwrap();
    // Real: `!is_empty` is true → returns "custom-editor".
    // Mutant (no !): `is_empty` is false → skips → falls to FALLBACK_EDITOR.
    assert_eq!(child_result, "custom-editor");
}

/// A stand-in editor that rewrites its file argument with fixed YAML, written
/// for the platform shell. `open_in_editor` appends the scratch path as the
/// trailing argument, which the script reads as `$1` / `%~1`.
///
/// One function with two `#[cfg]` bodies rather than two `#[cfg]`ed functions
/// of the same name: coverage tooling keys a function by (file, name), so a
/// same-named pair is one key with two entries, and `just crap::check` reported both
/// as new *and* both as removed on every run — against a baseline holding them
/// verbatim. A body-level `cfg` compiles to exactly the same thing on either
/// platform and leaves one key.
fn fake_editor(dir: &std::path::Path) -> String {
    #[cfg(not(windows))]
    {
        let script = dir.join("fake-editor.sh");
        std::fs::write(&script, "printf 'title: \"Edited\"\\n' > \"$1\"\n").unwrap();
        format!("sh {}", script.display())
    }
    #[cfg(windows)]
    {
        let script = dir.join("fake-editor.cmd");
        // No space before `>`: `echo x >f` would write a trailing space too.
        std::fs::write(&script, "@echo title: \"Edited\">\"%~1\"\r\n").unwrap();
        format!("cmd /C {}", script.display())
    }
}

#[test]
fn open_in_editor_runs_command_and_reads_back() {
    let dir = tempdir();
    let command = fake_editor(dir.path());
    let edited = open_in_editor(&command, "title:\n").unwrap();
    assert!(edited.contains("title: \"Edited\""), "{edited}");
}

#[test]
fn a_blank_editor_command_is_refused_with_where_to_set_one() {
    let err = open_in_editor("   ", "title:\n").unwrap_err();
    crate::diag::assert_plain(
        &err,
        "error: the editor command is empty\nhint: set [add] editor, $VISUAL or $EDITOR\n",
    );
}

#[test]
fn an_editor_that_cannot_start_names_the_command() {
    let err = open_in_editor("mt-no-such-editor-xyz", "title:\n").unwrap_err();
    let text = crate::diag::plain(&err);
    assert!(
        text.starts_with("error: can't start the editor: 'mt-no-such-editor-xyz': "),
        "{text}"
    );
}

/// A command that starts and exits non-zero, whatever the buffer path it is
/// given. One function with two body-level `#[cfg]`s, for the reason
/// [`fake_editor`] gives.
fn failing_editor() -> &'static str {
    #[cfg(not(windows))]
    {
        "false"
    }
    #[cfg(windows)]
    {
        "cmd /C exit 1"
    }
}

#[test]
fn an_editor_that_exits_non_zero_discards_the_buffer() {
    let command = failing_editor();
    let err = open_in_editor(command, "title:\n").unwrap_err();
    crate::diag::assert_plain(
        &err,
        &format!("error: editor exited without saving: '{command}'\n"),
    );
}

#[test]
fn never_mode_without_force_stays_closed() {
    let config = AddConfig {
        interactive: InteractiveMode::Never,
        ..Default::default()
    };
    // No terminal in tests; should_open must not error and must stay false.
    assert!(!should_open(false, &config, &specs(), &BTreeMap::new(), false).unwrap());
}

#[test]
fn always_mode_errors_without_tty() {
    let config = AddConfig {
        interactive: InteractiveMode::Always,
        ..Default::default()
    };
    assert!(should_open(false, &config, &specs(), &BTreeMap::new(), false).is_err());
}

#[test]
fn always_mode_succeeds_with_tty() {
    let config = AddConfig {
        interactive: InteractiveMode::Always,
        ..Default::default()
    };
    assert!(should_open(false, &config, &specs(), &BTreeMap::new(), true).unwrap());
}

// ── should_open: the negation that keeps a non-tty from opening an editor ──

#[test]
fn force_and_never_with_tty_opens() {
    let config = AddConfig {
        interactive: InteractiveMode::Never,
        ..Default::default()
    };
    let specs = discover_slots("#show: task.with(x: slot(\"x\"))");
    // force=true + Never + tty=true: real Ok(true).  Mutant (no !): Ok(false).
    assert!(should_open(true, &config, &specs, &BTreeMap::new(), true).unwrap());
}

// ── should_open / has_unfilled_required: the conjunctions deciding whether
//    a scaffold is worth opening, and the negation guarding each ──

#[test]
fn auto_mode_all_filled_stays_closed() {
    let config = AddConfig::default();
    let mut seed = BTreeMap::new();
    seed.insert("x".to_owned(), RawValue::Str("v".to_owned()));
    let specs = discover_slots("#show: task.with(x: slot(\"x\"))");
    // Seed fills the only required slot; real should_open = false.
    // Any of the four mutants makes should_open return true.
    assert!(!should_open(false, &config, &specs, &seed, true).unwrap());
}

// ── has_unfilled_required: a filled scaffold reports nothing outstanding ──

#[test]
fn auto_mode_unfilled_opens() {
    let config = AddConfig::default();
    let specs = discover_slots("#show: task.with(x: slot(\"x\"))");
    // No seed — has unfilled required; real should_open = true.
    // Mutant (return false) makes should_open = false.
    assert!(should_open(false, &config, &specs, &BTreeMap::new(), true).unwrap());
}

// ── has_unfilled_required: required AND unfilled, not either alone ──

#[test]
fn non_required_slot_not_unfilled() {
    let config = AddConfig::default();
    let specs = discover_slots("#show: task.with(x: slot(\"x\", required: false))");
    // spec not-required, no default, no seed → real has_unfilled_required = false.
    // Mutant: false || (true && true) = true → should_open = true.
    assert!(!should_open(false, &config, &specs, &BTreeMap::new(), true).unwrap());
}

// ── run: the edited buffer's values reach the caller ──

#[test]
fn run_parses_editor_output() {
    let dir = tempdir();
    let script = dir.path().join("editor.sh");
    std::fs::write(&script, "cat > \"$1\" << 'EOF'\nx: \"ParsedTitle\"\nEOF\n").unwrap();
    let editor = format!("bash {}", script.display());
    let specs = discover_slots("#show: task.with(x: slot(\"x\"))");
    let values = run(
        &AddConfig::default(),
        Some(&editor),
        &specs,
        &BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(values["x"], RawValue::Str("ParsedTitle".to_owned()));
}
