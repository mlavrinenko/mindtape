//! `mt go <task>` — open a task file in the configured editor.
//!
//! Resolves the task by handle (`1b`), slug, relative, or absolute path, then
//! spawns the `[go] editor` command. A `{task}` token in the command is replaced
//! with the resolved path; without it the path is appended. Editor resolution:
//! `[go] editor` → `$VISUAL` → `$EDITOR`, then error — no `vi` floor, so a GUI
//! user is told to configure `[go] editor` rather than dropped into `vi`.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::Args;
use mindtape::Config;

use crate::util;

/// Token in a `[go] editor` command replaced with the resolved task path.
const TASK_TOKEN: &str = "{task}";

/// Arguments for `mt go`.
#[derive(Args)]
pub struct GoArgs {
    /// Task to open: handle (`1b`), slug, relative path, or absolute path.
    pub id: String,
}

/// Run the `go` command.
///
/// # Errors
/// Returns an error if the task cannot be resolved, no editor is configured, the
/// editor command is empty, or the editor exits without success.
pub fn run(args: &GoArgs) -> Result<()> {
    let project = util::open()?;
    util::warn_errors(&project.outcome);

    let (task, abs) = util::resolve_task(&project, &args.id)?;

    let template = resolve_editor(&project.config)?;
    spawn_editor(&template, &abs)?;
    println!("opened {} ({})", args.id, task.title);
    Ok(())
}

/// Resolve the editor command template for a project: `[go] editor`, then
/// `$VISUAL`, then `$EDITOR`.
///
/// # Errors
/// Returns an error naming `[go] editor` when none of the three is set.
pub(crate) fn resolve_editor(config: &Config) -> Result<String> {
    editor_command(config.go.editor.as_deref())
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
    let (program, rest) = parts.split_first().context("empty editor command")?;

    let status = Command::new(program)
        .args(rest)
        .status()
        .with_context(|| format!("spawn editor {template:?}"))?;
    if !status.success() {
        bail!("editor {template:?} exited without success");
    }
    Ok(())
}

/// Resolve the editor command: `[go] editor`, then `$VISUAL`, then `$EDITOR`.
///
/// # Errors
/// Returns an error naming `[go] editor` when none of the three is set.
fn editor_command(configured: Option<&str>) -> Result<String> {
    if let Some(cmd) = configured
        && !cmd.trim().is_empty()
    {
        return Ok(cmd.trim().to_owned());
    }
    for var in ["VISUAL", "EDITOR"] {
        if let Ok(value) = std::env::var(var)
            && !value.trim().is_empty()
        {
            return Ok(value);
        }
    }
    bail!(
        "no editor configured: set `[go] editor` in .mindtape/config.toml (or $VISUAL / $EDITOR)"
    );
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
        bail!("empty editor command");
    }
    if !substituted {
        argv.push(path.to_owned());
    }
    Ok(argv)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{build_argv, editor_command};

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

    #[test]
    fn blank_configured_falls_through_to_error() {
        // No reliance on ambient $VISUAL/$EDITOR: a blank override plus the
        // common CI env (both unset) yields the configuration error.
        if std::env::var_os("VISUAL").is_none() && std::env::var_os("EDITOR").is_none() {
            assert!(editor_command(Some("  ")).is_err());
        }
    }
}
