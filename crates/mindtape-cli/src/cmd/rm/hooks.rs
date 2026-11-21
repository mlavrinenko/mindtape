//! `mt rm`'s `[[on.rm]]` action runner.
//!
//! `mt rm` is the only command that reaches this module. No read command does,
//! by construction: nothing in `ls`, `check`, `show` or `export` can reach
//! [`Hooks`].
//!
//! For one removal the runner walks the tasks this run removes in load order
//! and each matching hook's actions in declaration order. A `show` prints text
//! and a `list` prints a table. Both are presentation, so both fall silent when
//! stdout is not a terminal. A `run` executes a command. With `gate = true` a
//! non-zero exit stops the walk and refuses the removal, leaving every file
//! where it was. The command's output prints whatever the presentation setting:
//! it is the evidence the refusal rests on. `run` is optional on a gate. With
//! none, the hook's `when` already matching is the whole verdict, and the gate
//! refuses unconditionally with no command to run and nothing to print but its
//! `message`. `mt rm` has no `--force`, so a gate is final.
//!
//! No `run` command runs until the user allows the config's commands with
//! `mt allow` ([`mindtape::allow`]). Until then a gate refuses the removal as if
//! it had failed. A non-gating `run` skips with a line saying so. Either way the
//! command joins the held-back list that `main` warns about once.

use std::cell::OnceCell;
use std::collections::HashSet;
use std::io::Write;

use mindtape::config::hooks::{FlipAction, matching_rm_hooks};
use mindtape::shell::ConfigCommand;
use mindtape::{Readiness, Task};

use crate::cmd::hooks::{Presenter, RunAction, present};
use crate::diag::{Diag, Hint, Result};
use crate::util::Project;

/// The `[[on.rm]]` runner for one `mt rm` invocation.
///
/// It walks the tasks this run removes in load order and runs each matching
/// hook's actions in declaration order. A gate that fails refuses the whole
/// removal, and nothing reaches disk: the caller runs it before the prompt and
/// before it applies the plan.
pub(super) struct Hooks<'a> {
    project: &'a Project,
    /// The ids of the tasks this run removes.
    eligible: &'a HashSet<String>,
    readiness: &'a Readiness,
    /// Whether the runner renders `show` / `list` output at all.
    show: bool,
    /// Whether the user allowed the config's commands, asked on the first
    /// `run` only, since the answer costs a `git` call.
    allowed: OnceCell<bool>,
}

/// What a `run` action that fails without gating says when it doesn't declare
/// a `message`: the removal goes ahead, so the command is the user's to rerun.
const DEFAULT_ADVICE: &str = "fix it, then run the command yourself; the removal is not blocked";

/// Absolute path of the task the removal targets.
const ENV_FILE: &str = "MINDTAPE_TASK_FILE";
/// Identity of the task the removal targets (its file stem, unless `id:`
/// overrides).
const ENV_STEM: &str = "MINDTAPE_TASK_STEM";

impl<'a> Hooks<'a> {
    /// Build the runner for one removal: the project, the ids of the tasks it
    /// removes, the readiness their `when` filters read, and whether to render
    /// `show` / `list` output. A gate runs either way.
    pub(super) fn new(
        project: &'a Project,
        eligible: &'a HashSet<String>,
        readiness: &'a Readiness,
        show: bool,
    ) -> Self {
        Self {
            project,
            eligible,
            readiness,
            show,
            allowed: OnceCell::new(),
        }
    }

    /// Whether the user allowed the commands the project's config runs.
    fn allowed(&self) -> bool {
        *self.allowed.get_or_init(|| {
            let config = &self.project.config;
            mindtape::allow::is_allowed(&self.project.root, &mindtape::allow::commands(config))
        })
    }

    /// Run every action of every hook matching a removed task.
    ///
    /// # Errors
    /// Returns an error when a `when` or `list` expression fails to evaluate,
    /// an action doesn't declare a single kind, a command fails to spawn, or a
    /// gate refuses the removal.
    pub(super) fn run(&self, out: &mut dyn Write) -> Result<()> {
        if self.project.config.on.rm.is_empty() {
            return Ok(());
        }
        for task in &self.project.outcome.tasks {
            if !self.eligible.contains(&task.id) {
                continue;
            }
            for hook in matching_rm_hooks(&self.project.config, task, self.readiness)? {
                for action in &hook.actions {
                    self.act(task, action, out)?;
                }
            }
        }
        Ok(())
    }

    /// Run one action. A gate that fails returns the refusal, which stops the
    /// walk and the removal.
    fn act(&self, task: &Task, action: &FlipAction, out: &mut dyn Write) -> Result<()> {
        let presenter = Presenter::new(self.project, self.readiness, self.show);
        match present(action, "on.rm", &presenter, out)? {
            None => Ok(()),
            Some(run) => self.exec(task, run, out),
        }
    }

    /// Execute a `run` action and judge the result, or, with no command at all,
    /// refuse outright: a run-less gate has nothing to run, so `when` matching
    /// is the whole verdict.
    fn exec(&self, task: &Task, run: RunAction<'_>, out: &mut dyn Write) -> Result<()> {
        let Some(spec) = run.command else {
            return Err(refused(run.message));
        };
        let command = spec.to_string();
        if !self.allowed() {
            let project = self.project;
            mindtape::allow::hold(&project.root, &mindtape::allow::commands(&project.config));
            writeln!(out, "skipped: {command} (not allowed)")?;
            return if run.gate {
                Err(Diag::refused("gate not allowed")
                    .value(&command)
                    .hint(Hint::new("read the config's commands, then run mt allow")))
            } else {
                Ok(())
            };
        }
        let output = run_command(spec, self.project, task)?;
        let text = output.stdout + &output.stderr;
        if output.code == Some(0) {
            // A successful non-gate run's output is informational, suppressed
            // on the same terms as `show` and `list`.
            if !run.gate && self.show {
                write_output(out, &command, &text)?;
            }
            return Ok(());
        }
        // A failure always surfaces, quiet or not: it is either the evidence a
        // refusal rests on, or a command the project asked to be told about.
        write_output(out, &command, &text)?;
        if !run.gate {
            let advice = run.message.unwrap_or(DEFAULT_ADVICE);
            let verdict = verdict(output.code);
            writeln!(out, "warning: {command} ({verdict}) — {advice}")?;
            return Ok(());
        }
        let refusal = Diag::refused("gate failed")
            .value(&command)
            .exit(output.code);
        Err(match run.message {
            Some(message) => refusal.project_hint(message),
            None => refusal,
        })
    }
}

/// The run's outcome for a message: an exit code, or a signal.
fn verdict(code: Option<i32>) -> String {
    code.map_or_else(
        || "terminated by signal".to_owned(),
        |code| format!("exit {code}"),
    )
}

/// A run-less gate refuses unconditionally. `when` already matched, so there is
/// no command to run and nothing else to check. The `message`, when there is
/// one, is the refusal's explanation.
fn refused(message: Option<&str>) -> Diag {
    let refusal = Diag::refused("gate refused");
    match message {
        Some(message) => refusal.project_hint(message),
        None => refusal,
    }
}

/// Run `command` from the project root with the removed task in its
/// environment. The run captures its output rather than inheriting it. A gate's
/// evidence then prints under the refusal that rests on it.
///
/// # Errors
/// Returns an error when the shell itself fails to spawn. A command that runs
/// and fails is not an error here: its exit code says how it ended.
fn run_command(
    command: &ConfigCommand,
    project: &Project,
    task: &Task,
) -> Result<mindtape::shell::ShellOutput> {
    let path = project.root.join(&task.file);
    let env = [(ENV_FILE, path.as_os_str()), (ENV_STEM, task.id.as_ref())];
    mindtape::shell::run(command, &project.root, &env).map_err(|err| {
        Diag::failed("can't run on.rm command")
            .value(command.to_string())
            .io(&err)
    })
}

/// Print a command's combined output under a header naming it.
fn write_output(out: &mut dyn Write, command: &str, output: &str) -> Result<()> {
    writeln!(out, "$ {command}")?;
    for line in output.lines() {
        writeln!(out, "  {line}")?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "hooks_tests.rs"]
mod tests;
