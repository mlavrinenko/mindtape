//! `mt go <task>` — open a task file in the configured editor.
//!
//! Resolves the task by stem, stem hash, relative, or absolute path, then
//! spawns the `[go] editor` command. A `{task}` token in the command is replaced
//! with the resolved path; without it the path is appended. Editor resolution:
//! `[go] editor` → `$VISUAL` → `$EDITOR`, then error — no `vi` floor, so a GUI
//! user is told to configure `[go] editor` rather than dropped into `vi`. A
//! `[go] editor` the user has not allowed (`mt allow`) counts as unset, and
//! the end-of-run warning names it ([`mindtape::allow::runnable_editor`]).

use std::path::Path;
use std::process::Command;

use crate::diag::Result;
use clap::Args;
use mindtape::Config;

use crate::diag::{Command as DiagCommand, Diag, Hint};
use crate::util;

/// Token in a `[go] editor` command replaced with the resolved task path.
const TASK_TOKEN: &str = "{task}";

/// Arguments for `mt go`.
#[derive(Args)]
pub struct GoArgs {
    #[command(flatten)]
    pub task: crate::cli::task_ref::TaskRef,
}

/// Run the `go` command.
///
/// # Errors
/// Returns an error if the task cannot be resolved, no editor is configured, the
/// editor command is empty, or the editor exits without success.
pub fn run(args: &GoArgs) -> Result<()> {
    let project = util::open_warned()?;

    let (task, abs) = util::resolve_task(&project, &args.task.task)?;

    let template = resolve_editor(&project.root, &project.config)?;
    spawn_editor(&template, &abs)?;
    println!("opened {} ({})", args.task.task, task.title);
    Ok(())
}

/// Resolve the editor command template for the project at `root`:
/// `[go] editor` when the user allowed it, then `$VISUAL`, then `$EDITOR`.
///
/// # Errors
/// Returns an error naming `[go] editor` when none of the three is usable, or
/// pointing at `mt allow` when `[go] editor` is the one held back.
pub(crate) fn resolve_editor(root: &Path, config: &Config) -> Result<String> {
    let set = config.go.editor.as_deref().map(str::trim);
    let configured = mindtape::allow::runnable_editor(root, config, set);
    editor_command(configured.as_deref()).map_err(|err| match set {
        Some(held) if configured.is_none() && !held.is_empty() => held_editor(held),
        _ => err,
    })
}

/// The refusal for a `[go] editor` that waits for `mt allow`, with no
/// `$VISUAL` or `$EDITOR` to open instead.
fn held_editor(editor: &str) -> Diag {
    Diag::refused("the [go] editor waits for your approval")
        .value(editor)
        .hint(Hint::ending_in(
            "read it, then run",
            DiagCommand::mt().word("allow"),
        ))
}

/// Spawn the editor on `path` and wait for it to exit.
///
/// The `template` is split into an argv with `{task}` substituted (or `path`
/// appended when absent). Blocks until the editor returns so a terminal editor
/// holds the foreground; a GUI editor that forks returns immediately.
///
/// # Errors
/// Returns an error if the command is empty, the spawn fails, or the editor
/// exits without success.
pub(crate) fn spawn_editor(template: &str, path: &Path) -> Result<()> {
    let path = path.to_string_lossy();
    let parts = build_argv(template, &path)?;
    let (program, rest) = parts.split_first().ok_or_else(empty_editor)?;

    let status = Command::new(program).args(rest).status().map_err(|err| {
        Diag::failed("can't start the editor")
            .value(template)
            .io(&err)
    })?;
    if !status.success() {
        return Err(Diag::failed("the editor exited with an error").value(template));
    }
    Ok(())
}

/// Resolve the editor command: `[go] editor`, then `$VISUAL`, then `$EDITOR`.
///
/// # Errors
/// Returns an error naming `[go] editor` when none of the three is set.
fn editor_command(configured: Option<&str>) -> Result<String> {
    editor_from(configured, |var| std::env::var(var).ok())
}

/// [`editor_command`] with the environment injected, so each branch — blank
/// config falling through, a blank env value being skipped, nothing set at all
/// — is reachable from a test without touching the process environment.
fn editor_from(
    configured: Option<&str>,
    lookup: impl Fn(&str) -> Option<String>,
) -> Result<String> {
    if let Some(cmd) = configured
        && !cmd.trim().is_empty()
    {
        return Ok(cmd.trim().to_owned());
    }
    for var in ["VISUAL", "EDITOR"] {
        if let Some(value) = lookup(var)
            && !value.trim().is_empty()
        {
            return Ok(value);
        }
    }
    Err(Diag::refused("no editor configured").hint(Hint::new(
        "set [go] editor in .mindtape/config.toml, or $VISUAL or $EDITOR",
    )))
}

/// The refusal for an editor command with no program word in it.
fn empty_editor() -> Diag {
    Diag::refused("empty editor command").hint(Hint::example("vim {task}"))
}

/// Split `template` into an argv, substituting `{task}` with `path`. When no
/// token carries `{task}`, `path` is appended as the trailing argument.
///
/// # Errors
/// Returns an error if `template` holds no program word.
fn build_argv(template: &str, path: &str) -> Result<Vec<String>> {
    let mut argv = Vec::new();
    let mut substituted = false;
    for token in template.split_whitespace() {
        if token.contains(TASK_TOKEN) {
            substituted = true;
            argv.push(token.replace(TASK_TOKEN, path));
        } else {
            argv.push(token.to_owned());
        }
    }
    if argv.is_empty() {
        return Err(empty_editor());
    }
    if !substituted {
        argv.push(path.to_owned());
    }
    Ok(argv)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use mindtape::test_support::tempdir;

    use super::{build_argv, editor_command, editor_from};

    #[test]
    fn substitutes_task_token() {
        let argv = build_argv("zeditor {task}", "/p/a.typ").unwrap();
        assert_eq!(argv, vec!["zeditor", "/p/a.typ"]);
    }

    #[test]
    fn appends_path_when_no_token() {
        let argv = build_argv("nvim", "/p/a.typ").unwrap();
        assert_eq!(argv, vec!["nvim", "/p/a.typ"]);
    }

    #[test]
    fn keeps_other_args_and_appends() {
        let argv = build_argv("code --reuse-window", "/p/a.typ").unwrap();
        assert_eq!(argv, vec!["code", "--reuse-window", "/p/a.typ"]);
    }

    #[test]
    fn substitutes_token_glued_to_flag() {
        let argv = build_argv("editor --file={task}", "/p/a.typ").unwrap();
        assert_eq!(argv, vec!["editor", "--file=/p/a.typ"]);
    }

    #[test]
    fn token_present_does_not_also_append() {
        let argv = build_argv("e {task} --wait", "/p/a.typ").unwrap();
        assert_eq!(argv, vec!["e", "/p/a.typ", "--wait"]);
    }

    #[test]
    fn empty_template_errors() {
        assert!(build_argv("   ", "/p/a.typ").is_err());
    }

    #[test]
    fn configured_editor_wins() {
        assert_eq!(
            editor_command(Some("zeditor {task}")).unwrap(),
            "zeditor {task}"
        );
    }

    /// An environment with nothing set, for the cases that must not depend on
    /// whatever the developer happens to export.
    fn no_env(_: &str) -> Option<String> {
        None
    }

    /// A blank `[go] editor` is not a configured value: it falls through to the
    /// environment, and with nothing there it is an error naming the setting.
    /// The three tests this replaces each wrapped their assertion in an `if`
    /// over the ambient `$VISUAL`/`$EDITOR`, so on the wrong machine they
    /// asserted nothing at all.
    #[test]
    fn blank_config_and_empty_env_is_an_error() {
        for blank in ["", "  "] {
            let err = crate::diag::plain(&editor_from(Some(blank), no_env).unwrap_err());
            assert!(
                err.starts_with("error: no editor configured\nhint: set [go] editor"),
                "the error must name the setting to fix: {err}"
            );
        }
    }

    /// `editor_command` with a valid non-blank config returns the command.
    /// The `delete !` mutant would NOT take the `&& !is_empty` branch and would
    /// fall through to env vars.
    #[test]
    fn editor_command_uses_configured_value() {
        assert_eq!(
            editor_command(Some("unlikely-editor")).unwrap(),
            "unlikely-editor"
        );
    }

    /// With no configured value the environment supplies the editor, and
    /// `$VISUAL` wins over `$EDITOR` — the precedence that decides which of a
    /// developer's two editors actually opens.
    #[test]
    fn visual_wins_over_editor() {
        let both = |var: &str| Some(format!("{var}-editor").to_lowercase());
        assert_eq!(editor_from(None, both).unwrap(), "visual-editor");
    }

    /// A blank `$VISUAL` is skipped rather than returned, so `$EDITOR` still
    /// gets its turn. Under the `delete !` mutant on the env check, the blank
    /// would be accepted and returned verbatim.
    #[test]
    fn blank_visual_falls_through_to_editor() {
        let lookup = |var: &str| match var {
            "VISUAL" => Some("   ".to_owned()),
            _ => Some("nano".to_owned()),
        };
        assert_eq!(editor_from(None, lookup).unwrap(), "nano");
    }

    /// A task path under a tempdir. `spawn_editor` only ever passes it to the
    /// editor as an argument, but a shared literal like `/tmp/x` is a path
    /// another process on the machine can own, so give each test its own.
    fn task_path(dir: &tempfile::TempDir) -> std::path::PathBuf {
        dir.path().join("task.typ")
    }

    /// `spawn_editor` with a broken template (empty program) returns an error.
    /// A body replaced with `Ok(())` would succeed silently.
    #[test]
    fn spawn_editor_errors_on_empty_template() {
        let dir = tempdir();
        assert!(super::spawn_editor("   ", &task_path(&dir)).is_err());
    }

    /// A command that always exits 0, whatever the platform: `true` is a POSIX
    /// utility Windows has no counterpart for, so `cmd /C rem` (a comment)
    /// stands in. `spawn_editor` appends the task path either way, and both
    /// ignore it.
    #[cfg(not(windows))]
    const ALWAYS_OK: &str = "true";
    #[cfg(windows)]
    const ALWAYS_OK: &str = "cmd /C rem";

    /// `spawn_editor` with a valid command succeeds. The `delete !` mutant
    /// would invert the success check, making a successful editor `bail!`.
    #[test]
    fn spawn_editor_succeeds_with_true() {
        // The `delete !` mutant would check `if true` and bail.
        let dir = tempdir();
        super::spawn_editor(ALWAYS_OK, &task_path(&dir)).unwrap();
    }
}
