#![allow(clippy::unwrap_used)]

use mindtape::test_support::tempdir;
use mindtape::test_support::write_file;
use std::path::Path;

use super::{
    HOOKS, completion_hint, completions_dir, quote_path, shell_from_env, write_completions,
    write_defaults, write_schema, xdg_schema_path,
};

/// A file installer under test: `(dir, force) -> Result`.
type Writer = fn(&Path, bool) -> crate::diag::Result<()>;

/// `writer` creates `dir/name` with exactly `expected` when absent.
fn assert_writes_content(name: &str, writer: Writer, expected: &str) {
    let dir = tempdir();
    let target = dir.path().join("mindtape");
    writer(&target, false).unwrap();
    assert_eq!(
        std::fs::read_to_string(target.join(name)).unwrap(),
        expected
    );
}

/// An existing `dir/name` survives `writer` without `force`.
fn assert_skips_existing(name: &str, writer: Writer) {
    let dir = tempdir();
    let target = dir.path().join("mindtape");
    std::fs::create_dir_all(&target).unwrap();
    write_file(&target, name, "# existing");
    writer(&target, false).unwrap();
    assert_eq!(
        std::fs::read_to_string(target.join(name)).unwrap(),
        "# existing"
    );
}

/// `force` overwrites an existing `dir/name` with exactly `expected`.
fn assert_overwrites(name: &str, writer: Writer, expected: &str) {
    let dir = tempdir();
    let target = dir.path().join("mindtape");
    std::fs::create_dir_all(&target).unwrap();
    write_file(&target, name, "# old");
    writer(&target, true).unwrap();
    assert_eq!(
        std::fs::read_to_string(target.join(name)).unwrap(),
        expected
    );
}

#[test]
fn write_defaults_creates_file_when_absent() {
    let dir = tempdir();
    let target = dir.path().join("mindtape");
    write_defaults(&target, false).unwrap();
    assert!(target.join("defaults.toml").exists());
}

#[test]
fn write_defaults_content_matches_embedded_floor() {
    assert_writes_content(
        "defaults.toml",
        write_defaults,
        mindtape::embedded_defaults(),
    );
}

#[test]
fn write_defaults_skips_existing_without_force() {
    assert_skips_existing("defaults.toml", write_defaults);
}

#[test]
fn write_defaults_overwrites_with_force() {
    assert_overwrites(
        "defaults.toml",
        write_defaults,
        mindtape::embedded_defaults(),
    );
}

#[test]
fn write_schema_content_matches_generated_schema() {
    assert_writes_content(
        "config.schema.json",
        write_schema,
        &mindtape::config_schema_json(),
    );
}

#[test]
fn write_schema_skips_existing_without_force() {
    assert_skips_existing("config.schema.json", write_schema);
}

#[test]
fn write_schema_overwrites_with_force() {
    assert_overwrites(
        "config.schema.json",
        write_schema,
        &mindtape::config_schema_json(),
    );
}

/// The `--no-schema` half writes the `at.toml` schema beside the config one,
/// so an `at.toml` can point its `#:schema` line at `./at.schema.json`.
#[test]
fn write_schema_writes_the_at_schema_beside_it() {
    assert_writes_content("at.schema.json", write_schema, &mindtape::at::schema_json());
}

#[test]
fn write_schema_skips_an_existing_at_schema_without_force() {
    assert_skips_existing("at.schema.json", write_schema);
}

#[test]
fn write_schema_overwrites_the_at_schema_with_force() {
    assert_overwrites("at.schema.json", write_schema, &mindtape::at::schema_json());
}

/// `xdg_schema_path` returns a path ending with `config.schema.json`,
/// not a `Default::default()` (empty) `PathBuf`.
#[test]
fn xdg_schema_path_ends_with_config_schema_json() {
    let path = xdg_schema_path().unwrap();
    assert!(path.ends_with("config.schema.json"));
}

/// Regression: the seeded floor text parses and validates on its own — the
/// same guarantee `cmd/init/tests.rs` asserts for the slim/full project scaffolds.
#[test]
fn seeded_defaults_parses_and_validates() {
    let config = mindtape::parse_config(mindtape::embedded_defaults()).unwrap();
    config.validate().unwrap();
}

/// `completions_dir` composes `<config home>/mindtape/completions`, not an
/// empty `Default::default()` path.
#[test]
fn completions_dir_ends_with_the_completions_folder() {
    let path = completions_dir().unwrap();
    assert_eq!(
        path.file_name().and_then(std::ffi::OsStr::to_str),
        Some("completions"),
        "{path:?}"
    );
    assert!(path.to_string_lossy().contains("mindtape"), "{path:?}");
}

/// `write_completions` actually writes every hook `HOOKS` declares — a no-op
/// leaves a shell that installed completions pointing at nothing.
#[test]
fn write_completions_writes_a_hook_per_shell() {
    let dir = tempdir();
    write_completions(dir.path(), false).unwrap();
    for (shell, hook) in HOOKS {
        assert_eq!(
            std::fs::read_to_string(dir.path().join(shell)).unwrap(),
            *hook,
            "{shell}"
        );
    }
}

/// Env marker gating the inner side of the `run` completion tests below.
const RUN_COMPLETIONS_INNER: &str = "_MT_INSTALL_RUN_COMPLETIONS";

/// Re-run the calling test in a child with the config/data homes pointed at
/// `home`.
fn run_install_inner(home: &Path) {
    let out = mindtape::test_support::rerun_isolated_with(RUN_COMPLETIONS_INNER, |child| {
        child
            .env(mindtape::CONFIG_HOME_ENV, home.join("config"))
            .env(mindtape::DATA_HOME_ENV, home.join("data"));
    });
    assert!(out.is_some(), "only the parent runs the child");
}

/// A bare `install` writes the completion hooks. Pins the `!` in
/// `if !args.no_completions`: the inverted guard skips them.
#[test]
fn run_writes_completion_hooks_by_default() {
    if std::env::var(RUN_COMPLETIONS_INNER).is_ok() {
        super::run(&super::InstallArgs {
            no_lib: true,
            no_defaults: true,
            no_schema: true,
            no_completions: false,
            force: false,
        })
        .unwrap();
        return;
    }
    let home = tempdir();
    run_install_inner(home.path());
    assert!(
        home.path()
            .join("config/mindtape/completions/bash")
            .exists()
    );
}

/// `--no-completions` is the only thing that skips them.
#[test]
fn run_with_no_completions_writes_no_hooks() {
    if std::env::var(RUN_COMPLETIONS_INNER).is_ok() {
        super::run(&super::InstallArgs {
            no_lib: true,
            no_defaults: true,
            no_schema: true,
            no_completions: true,
            force: false,
        })
        .unwrap();
        return;
    }
    let home = tempdir();
    run_install_inner(home.path());
    assert!(
        !home
            .path()
            .join("config/mindtape/completions/bash")
            .exists()
    );
}

/// `$SHELL` picks the hook the hint shows, by its last path segment on either
/// separator and without `.exe`, so a Windows path reads the same on any host.
/// A shell with no hook of its own, or no `$SHELL` at all, gets bash's line.
#[test]
fn shell_from_env_names_the_hook_for_shell() {
    let cases = [
        (Some("/usr/bin/zsh"), "zsh"),
        (Some("/run/current-system/sw/bin/fish"), "fish"),
        (Some("elvish"), "elvish"),
        (Some("/usr/bin/pwsh"), "powershell"),
        (
            Some("C:\\Program Files\\PowerShell\\7\\pwsh.exe"),
            "powershell",
        ),
        (Some("C:\\msys64\\usr\\bin\\bash.exe"), "bash"),
        (Some("/bin/sh"), "bash"),
        (Some(""), "bash"),
        (None, "bash"),
    ];
    for (shell, hook) in cases {
        assert_eq!(shell_from_env(shell), hook, "{shell:?}");
    }
}

/// Each shell gets the line its own startup file takes.
#[test]
fn completion_hint_shows_the_source_line_for_the_shell() {
    let dir = Path::new("/home/u/.config/mindtape/completions");
    // Through `quote_path`: on Windows `join` adds a backslash, which quotes.
    let path = |shell: &str| quote_path(&dir.join(shell));
    assert_eq!(
        completion_hint("bash", dir),
        format!(
            "For Tab completion, add this to your shell's startup file:\n  source {}",
            path("bash")
        )
    );
    let cases = [
        ("zsh", format!("source {}", path("zsh"))),
        ("fish", format!("source {}", path("fish"))),
        ("elvish", format!("eval (slurp < {})", path("elvish"))),
        (
            "powershell",
            format!(
                "Get-Content -Raw {} | Invoke-Expression",
                path("powershell")
            ),
        ),
    ];
    for (shell, line) in cases {
        let hint = completion_hint(shell, dir);
        assert!(hint.ends_with(&format!("\n  {line}")), "{hint}");
    }
}

/// A path with a space or a backslash, as on Windows, is quoted so the line
/// still pastes as one argument.
#[test]
fn completion_hint_quotes_a_path_that_needs_it() {
    let line = completion_hint("bash", Path::new("/home/a user/completions"));
    let quoted = format!(
        "'{}'",
        Path::new("/home/a user/completions").join("bash").display()
    );
    assert!(line.ends_with(&format!("source {quoted}")), "{line}");
}
