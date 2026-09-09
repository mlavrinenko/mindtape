//! `mt flip`'s `[[on.flip]]` action runner.
//!
//! `mt flip` is the only command that reaches this module — no read command
//! does, by construction: nothing in `ls`, `check`, `show` or `export` can
//! reach [`Hooks`], and `flip_hooks_e2e.rs` pins that with a hook whose command
//! would leave a file behind.
//!
//! For one transition the runner walks the matching hooks' actions in
//! declaration order. A `show` prints text and a `list` prints a table; both are
//! presentation, so both fall silent under `--quiet` or when stdout is not a
//! terminal. A `run` executes a command; with `gate = true` a non-zero exit
//! stops the walk and refuses the flip, so nothing is written for that task, and
//! the command's output is printed whatever the presentation setting — it is the
//! evidence the refusal rests on. `--force` waives a failing gate and records
//! the waiver in the status note ([`note_with_overrides`]), because an override
//! nobody can see afterwards is not an override, it is a hole.

use std::io::Write;
use std::path::Path;

use anyhow::{Result, anyhow};
use mindtape::config_hooks::{ActionKind, FlipAction, matching_flip_hooks};
use mindtape::{ConditionEnv, Filter, Readiness, Task, select};

use crate::flip_hooks_cmd::{CommandReport, RunCtx, run_command};
use crate::output::{OutputFormat, render};
use crate::render::RenderCtx;
use crate::util::Project;

/// Advice attached to a gate refusal the action declares no `message` for.
const DEFAULT_ADVICE: &str = "fix it, or re-run with --force (the override is \
                              recorded in the status note)";

/// One transition, as the hooks see it.
pub(crate) struct Transition<'a> {
    /// The task as it stands before the write.
    pub task: &'a Task,
    /// Absolute path of its file.
    pub path: &'a Path,
    /// The status being stamped.
    pub target: &'a str,
    /// Whether `--force` waives a failing gate.
    pub force: bool,
}

/// What running one transition's hooks concluded.
#[derive(Debug, Default)]
pub(crate) struct HookOutcome {
    /// Gate commands that failed and `--force` waived, in run order.
    pub forced: Vec<String>,
    /// Set when a gate failed and was not waived: why the flip is refused.
    pub refusal: Option<String>,
}

/// A gate command that exited non-zero.
struct Gate {
    /// The command line, for the status note.
    command: String,
    /// The refusal message, for the caller's problem report.
    refusal: String,
}

/// The `run` action's fields, grouped so the executor stays inside the
/// argument-count lint.
#[derive(Clone, Copy)]
struct RunSpec<'a> {
    command: &'a str,
    gate: bool,
    message: Option<&'a str>,
}

/// The `[[on.flip]]` runner for one `mt flip` invocation.
pub(crate) struct Hooks<'a> {
    project: &'a Project,
    readiness: Readiness,
    /// Whether `show` / `list` output is rendered at all.
    show: bool,
    /// The environment a `list` table's column visibility resolves against —
    /// built exactly as `mt ls` builds it, so a `[[ls.columns.<name>.rules]]`
    /// condition means the same thing in both surfaces.
    env: ConditionEnv,
}

impl<'a> Hooks<'a> {
    /// Build the runner. `show` is `false` under `--quiet` or when stdout is not
    /// a terminal; gates run either way.
    ///
    /// Readiness is computed once, over the whole loaded set, so a `when` may
    /// use `ready` / `blocked` — and is skipped entirely when the project
    /// declares no hooks, which is the common case.
    pub(crate) fn new(project: &'a Project, show: bool) -> Self {
        let readiness = if project.config.on.flip.is_empty() {
            Readiness::empty()
        } else {
            Readiness::compute(&project.outcome.tasks, &project.config)
        };
        Self {
            project,
            readiness,
            show,
            env: crate::ls_rules::build_env(project, "table"),
        }
    }

    /// Run every action of every hook matching `at`, in declaration order,
    /// stopping at the first gate that fails without `--force`.
    ///
    /// # Errors
    /// Returns an error when a `when` or `list` expression cannot be evaluated,
    /// an action declares no single kind, a command cannot be spawned, or
    /// writing to `out` fails.
    pub(crate) fn run(&self, at: &Transition<'_>, out: &mut dyn Write) -> Result<HookOutcome> {
        let mut outcome = HookOutcome::default();
        let hooks = matching_flip_hooks(&self.project.config, at.task, at.target, &self.readiness)?;
        for hook in hooks {
            for action in &hook.actions {
                let Some(gate) = self.act(action, at, out)? else {
                    continue;
                };
                if !at.force {
                    outcome.refusal = Some(gate.refusal);
                    return Ok(outcome);
                }
                writeln!(out, "--force: proceeding past the failing gate")?;
                outcome.forced.push(gate.command);
            }
        }
        Ok(outcome)
    }

    /// Run one action, returning the gate it failed, if any.
    fn act(
        &self,
        action: &FlipAction,
        at: &Transition<'_>,
        out: &mut dyn Write,
    ) -> Result<Option<Gate>> {
        match action
            .kind()
            .map_err(|message| anyhow!("on.flip action: {message}"))?
        {
            ActionKind::Show(text) => {
                if self.show {
                    writeln!(out, "{text}")?;
                }
                Ok(None)
            }
            ActionKind::List { filter, title } => {
                self.list(filter, title, out)?;
                Ok(None)
            }
            ActionKind::Run {
                command,
                gate,
                message,
            } => self.exec(
                RunSpec {
                    command,
                    gate,
                    message,
                },
                at,
                out,
            ),
        }
    }

    /// Print the tasks matching `spec` (a filter expression or `@view`) as a
    /// table, under `title`. A bare `@view` also picks up that view's sort.
    fn list(&self, spec: &str, title: Option<&str>, out: &mut dyn Write) -> Result<()> {
        if !self.show {
            return Ok(());
        }
        let config = &self.project.config;
        let filter = Filter::parse(spec)?;
        let mut tasks = select(
            self.project.outcome.tasks.clone(),
            Some(&filter),
            config,
            &self.readiness,
        )?;
        crate::ls::sort_tasks(
            &mut tasks,
            &view_sort(self.project, spec),
            config,
            &self.readiness,
        )?;
        if let Some(title) = title {
            writeln!(out, "{title}")?;
        }
        if tasks.is_empty() {
            writeln!(out, "  (no matching tasks)")?;
            return Ok(());
        }
        let ctx = RenderCtx {
            config,
            readiness: &self.readiness,
            env: &self.env,
        };
        let table = render(&tasks, OutputFormat::Table, ctx, None)?;
        writeln!(out, "{table}")?;
        Ok(())
    }

    /// Execute a `run` action and judge the result.
    fn exec(
        &self,
        run: RunSpec<'_>,
        at: &Transition<'_>,
        out: &mut dyn Write,
    ) -> Result<Option<Gate>> {
        let report = run_command(
            run.command,
            &RunCtx {
                root: &self.project.root,
                path: at.path,
                stem: &at.task.id,
                target: at.target,
            },
        )?;
        if report.success {
            // A successful non-gate run's output is informational — presentation,
            // suppressed on the same terms as `show` and `list`.
            if !run.gate && self.show {
                write_output(out, run.command, &report)?;
            }
            return Ok(None);
        }
        // A failure always surfaces, quiet or not: it is either the evidence a
        // refusal rests on, or a command the project asked to be told about.
        write_output(out, run.command, &report)?;
        let advice = run.message.unwrap_or(DEFAULT_ADVICE);
        let verdict = report.verdict();
        if !run.gate {
            writeln!(out, "warning: {} ({verdict}) — {advice}", run.command)?;
            return Ok(None);
        }
        Ok(Some(Gate {
            command: run.command.to_owned(),
            refusal: format!(
                "gate {command:?} failed ({verdict}); nothing written — {advice}",
                command = run.command
            ),
        }))
    }
}

/// The sort chain a `list` spec inherits: a bare `@view` reference brings its
/// view's sort, anything else takes the default order.
fn view_sort(project: &Project, spec: &str) -> Vec<String> {
    spec.strip_prefix('@')
        .and_then(|name| project.config.views.get(name))
        .map(|view| view.sort.clone())
        .unwrap_or_default()
}

/// Print a command's combined output under a header naming it.
fn write_output(out: &mut dyn Write, command: &str, report: &CommandReport) -> Result<()> {
    writeln!(out, "$ {command}")?;
    for line in report.output.lines() {
        writeln!(out, "  {line}")?;
    }
    Ok(())
}

/// Fold every `--force`-waived gate into the status note.
///
/// A waived gate that leaves no trace is indistinguishable afterwards from a
/// gate that passed, which would make the whole mechanism advisory. The note is
/// where a status already explains itself, so the waiver goes there — appended
/// to whatever `-m` supplied, or standing alone when it supplied nothing.
pub(crate) fn note_with_overrides(note: Option<&str>, forced: &[String]) -> Option<String> {
    if forced.is_empty() {
        return note.map(str::to_owned);
    }
    let commands: Vec<String> = forced.iter().map(|cmd| format!("{cmd:?}")).collect();
    let marker = format!("forced past failing gate: {}", commands.join(", "));
    Some(match note.map(str::trim).filter(|text| !text.is_empty()) {
        Some(text) => format!("{text} — {marker}"),
        None => marker,
    })
}

#[cfg(test)]
#[path = "flip_hooks_tests.rs"]
mod tests;
