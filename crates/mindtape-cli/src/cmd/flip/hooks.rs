//! `mt flip`'s `[[on.flip]]` action runner.
//!
//! `mt flip` is the only command that reaches this module. No read command
//! does, by construction: nothing in `ls`, `check`, `show` or `export` can
//! reach [`Hooks`], and `flip_hooks_e2e.rs` pins that with a hook whose command
//! would leave a file behind.
//!
//! For one transition the runner walks the matching hooks' actions in
//! declaration order. A `show` prints text and a `list` prints a table. Both are
//! presentation, so both fall silent under `--quiet` or when stdout is not a
//! terminal. A `run` executes a command. With `gate = true` a non-zero exit
//! stops the walk and refuses the flip, leaving that task's file unwritten. The
//! command's output prints whatever the presentation setting: it is the
//! evidence the refusal rests on. `run` is optional on a gate. With none, the
//! hook's `when` already matching is the whole verdict, and the gate refuses
//! unconditionally with no command to run and nothing to print but its
//! `message`. `--force` waives a failing gate, run or run-less, and records the
//! waiver in the status note ([`note_with_overrides`]). An override nobody can
//! see afterwards is not an override, it is a hole.
//!
//! Once the batch's writes have landed, [`Hooks::after`] runs the `after`
//! actions of every hook that matched a landed target, once for the batch, in
//! declaration order. That phase cannot refuse anything. The write already
//! happened. A failing command prints its output and a warning, and the command
//! exits zero. It is where a project refreshes what its own write staled.
//!
//! No `run` command runs until the user allows the config's commands with
//! `mt allow` ([`mindtape::allow`]). Until then a gate refuses the flip as if
//! it had failed, so `--force` waives it the same way. A non-gating `run` skips
//! with a line saying so, and so does an after-the-write `run`. Either way the
//! command joins the held-back list that `main` warns about once.

use std::cell::OnceCell;
use std::io::Write;
use std::path::Path;

use crate::diag::Result;
use mindtape::config::hooks::{FlipAction, matching_flip_hooks};
use mindtape::shell::ConfigCommand;
use mindtape::{Readiness, Task};

use crate::cmd::flip::batch::Landed;
use crate::cmd::flip::chain::Chain;
use crate::cmd::flip::hooks_cmd::{
    BatchCtx, CommandReport, RunCtx, run_batch_command, run_command,
};
use crate::cmd::hooks::{Presenter, RunAction, present};
use crate::diag::{Diag, Hint};
use crate::util::Project;

/// The advice a gate keeps when `--force` can still waive it: what a run-less
/// gate without a `message` records in the status note.
const GATE_ADVICE: &str = "fix it, or re-run with --force (the override is \
                           recorded in the status note)";

/// What a `run` action that fails without gating says when it doesn't declare
/// a `message`: the flip goes ahead, so the command is the user's to rerun.
const RUN_ADVICE: &str = "fix it, then run the command yourself; the flip is not blocked";

/// What an after-the-write `run` that fails says when it doesn't declare a
/// `message`: the write already landed, so the command is the user's to rerun.
const AFTER_ADVICE: &str = "fix it, then run the command yourself; the write already landed";

/// One transition, as the hooks see it.
pub(crate) struct Transition<'a> {
    /// The task as it stands before the write.
    pub task: &'a Task,
    /// Absolute path of its file.
    pub path: &'a Path,
    /// The status the flip stamps.
    pub target: &'a str,
    /// Whether `--force` waives a failing gate.
    pub force: bool,
}

/// What running one transition's hooks concluded.
#[derive(Debug, Default)]
pub(crate) struct HookOutcome {
    /// Gate commands that failed and `--force` waived, in run order.
    pub forced: Vec<String>,
    /// Set when a gate failed and `--force` did not waive it: the reason the flip
    /// fails.
    pub refusal: Option<Diag>,
    /// Positions in `config.on.flip` of the hooks whose `when` matched, in
    /// declaration order. The batch remembers them for the landed targets, so
    /// each hook's `after` actions run once.
    pub matched: Vec<usize>,
}

/// A gate that refused the flip: a `run` command that exited non-zero, or a
/// run-less gate whose `when` matched.
struct Gate {
    /// The command line, for the status note, or, for a run-less gate with no
    /// command to name, its `message` (falling back to [`GATE_ADVICE`]).
    command: String,
    /// The refusal, for the caller's problem report.
    refusal: Diag,
}

/// The `[[on.flip]]` runner for one `mt flip` invocation.
pub(crate) struct Hooks<'a> {
    project: &'a Project,
    readiness: Readiness,
    /// Whether the runner renders `show` / `list` output at all.
    show: bool,
    /// Whether the user allowed the config's commands, asked on the first
    /// `run` only, since the answer costs a `git` call.
    allowed: OnceCell<bool>,
    /// The flips in progress above this `mt`, which every `run` command gets
    /// with its own flip added.
    chain: Chain,
}

impl<'a> Hooks<'a> {
    /// Build the runner. `show` is `false` under `--quiet` or when stdout is not
    /// a terminal. Gates run either way.
    ///
    /// The runner computes readiness once, over the whole loaded set, so a
    /// `when` may use `ready` / `blocked`. It skips the computation entirely
    /// when the project doesn't declare any hooks, which is the common case.
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
            allowed: OnceCell::new(),
            chain: Chain::inherited(),
        }
    }

    /// The flips in progress above this `mt`.
    pub(crate) fn chain(&self) -> &Chain {
        &self.chain
    }

    /// Whether the user allowed the commands the project's config runs.
    fn allowed(&self) -> bool {
        *self.allowed.get_or_init(|| {
            let commands = mindtape::allow::commands(&self.project.config);
            mindtape::allow::is_allowed(&self.project.root, &commands)
        })
    }

    /// Whether a command may run, recording the config's commands as held back
    /// when it may not. The record is what `main`'s one warning lists.
    fn allowed_or_hold(&self) -> bool {
        if self.allowed() {
            return true;
        }
        let project = &self.project;
        mindtape::allow::hold(&project.root, &mindtape::allow::commands(&project.config));
        false
    }

    /// Run every action of every hook matching `at`, in declaration order,
    /// stopping at the first gate that fails without `--force`.
    ///
    /// # Errors
    /// Returns an error when a `when` or `list` expression fails to evaluate,
    /// an action doesn't declare a single kind, a command fails to spawn, or
    /// writing to `out` fails.
    pub(crate) fn run(&self, at: &Transition<'_>, out: &mut dyn Write) -> Result<HookOutcome> {
        let mut outcome = HookOutcome::default();
        let hooks = matching_flip_hooks(&self.project.config, at.task, at.target, &self.readiness)?;
        for (index, hook) in hooks {
            outcome.matched.push(index);
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

    /// Run the after-the-write actions of the hooks at `matched`, once for the
    /// batch, in declaration order. `target` is the status the batch stamped,
    /// and `landed` the tasks it wrote.
    ///
    /// Nothing here can refuse the flip: the write already happened. A command
    /// that fails prints its output and a warning, and the flip still exits
    /// zero.
    ///
    /// # Errors
    /// Returns an error when a `list` expression fails to evaluate, an action
    /// doesn't declare a single kind, or writing to `out` fails.
    pub(crate) fn after(
        &self,
        matched: &[usize],
        landed: &Landed,
        target: &str,
        out: &mut dyn Write,
    ) -> Result<()> {
        let hooks = matched
            .iter()
            .filter_map(|index| self.project.config.on.flip.get(*index));
        for hook in hooks {
            for action in &hook.after {
                self.act_after(action, landed, target, out)?;
            }
        }
        Ok(())
    }

    /// Run one after-the-write action. A `show` or `list` presents as it does
    /// before the write. A `run` executes and warns on failure.
    fn act_after(
        &self,
        action: &FlipAction,
        landed: &Landed,
        target: &str,
        out: &mut dyn Write,
    ) -> Result<()> {
        let presenter = Presenter::new(self.project, &self.readiness, self.show);
        let ctx = BatchCtx {
            root: &self.project.root,
            target,
            chain: &self.chain,
            landed,
        };
        match present(action, "on.flip", &presenter, out)? {
            Some(RunAction {
                command: Some(spec),
                message,
                ..
            }) => self.exec_after(spec, message, &ctx, out),
            // A `show` or `list`, or a run-less action, which config load
            // already refuses in an `after` list.
            _ => Ok(()),
        }
    }

    /// Execute an after-the-write `run` and report what it did.
    fn exec_after(
        &self,
        spec: &ConfigCommand,
        message: Option<&str>,
        ctx: &BatchCtx<'_>,
        out: &mut dyn Write,
    ) -> Result<()> {
        let command = spec.to_string();
        if !self.allowed_or_hold() {
            writeln!(out, "skipped: {command} (not allowed)")?;
            return Ok(());
        }
        let report = match run_batch_command(spec, ctx) {
            Ok(report) => report,
            Err(err) => {
                writeln!(out, "warning: {command} could not run — {err}")?;
                return Ok(());
            }
        };
        if report.success {
            // A passing after command's output is informational, suppressed on
            // the same terms as `show` and `list`.
            if self.show {
                write_output(out, &command, &report)?;
            }
            return Ok(());
        }
        // A failure always surfaces: the project asked to be told about it.
        write_output(out, &command, &report)?;
        let advice = message.unwrap_or(AFTER_ADVICE);
        writeln!(out, "warning: {command} ({}) — {advice}", report.verdict())?;
        Ok(())
    }

    /// Run one action, returning the gate it failed, if any.
    fn act(
        &self,
        action: &FlipAction,
        at: &Transition<'_>,
        out: &mut dyn Write,
    ) -> Result<Option<Gate>> {
        let presenter = Presenter::new(self.project, &self.readiness, self.show);
        match present(action, "on.flip", &presenter, out)? {
            None => Ok(None),
            Some(run) => self.exec(run, at, out),
        }
    }

    /// Execute a `run` action and judge the result, or, with no command at all,
    /// refuse outright: a run-less gate has nothing to run, so `when` matching
    /// is the whole verdict.
    fn exec(
        &self,
        run: RunAction<'_>,
        at: &Transition<'_>,
        out: &mut dyn Write,
    ) -> Result<Option<Gate>> {
        let Some(spec) = run.command else {
            return Ok(Some(runless_gate(run.message)));
        };
        let command = spec.to_string();
        if !self.allowed_or_hold() {
            return held_back(&command, run.gate, out);
        }
        let report = run_command(
            spec,
            &RunCtx {
                root: &self.project.root,
                path: at.path,
                stem: &at.task.id,
                target: at.target,
                chain: &self.chain,
            },
        )?;
        if report.success {
            // A successful non-gate run's output is informational — presentation,
            // suppressed on the same terms as `show` and `list`.
            if !run.gate && self.show {
                write_output(out, &command, &report)?;
            }
            return Ok(None);
        }
        // A failure always surfaces, quiet or not: it is either the evidence a
        // refusal rests on, or a command the project asked to be told about.
        write_output(out, &command, &report)?;
        if !run.gate {
            let advice = run.message.unwrap_or(RUN_ADVICE);
            writeln!(out, "warning: {command} ({}) — {advice}", report.verdict())?;
            return Ok(None);
        }
        let refusal = Diag::refused("gate failed")
            .value(&command)
            .exit(report.code);
        Ok(Some(Gate {
            command,
            refusal: with_override_hint(refusal, run.message),
        }))
    }
}

/// The verdict on a `run` the user has not allowed. The runner says it skipped
/// the command, and refuses the flip when it is a gate, since a check that never
/// ran is no evidence at all.
fn held_back(command: &str, gate: bool, out: &mut dyn Write) -> Result<Option<Gate>> {
    writeln!(out, "skipped: {command} (not allowed)")?;
    if !gate {
        return Ok(None);
    }
    let refusal = Diag::refused("gate not allowed")
        .value(command)
        .hint(Hint::new("read the config's commands, then run mt allow"))
        .hint(Hint::new(
            "or pass --force to skip the gate; the status note records it",
        ));
    Ok(Some(Gate {
        command: command.to_owned(),
        refusal,
    }))
}

/// A run-less gate refuses unconditionally. `when` already matched, so there is
/// no command to run and nothing else to check. The `message` (or the default
/// advice, absent one) is both the refusal's explanation and the stand-in for a
/// command in the status note.
fn runless_gate(message: Option<&str>) -> Gate {
    let advice = message.unwrap_or(GATE_ADVICE);
    Gate {
        command: advice.to_owned(),
        refusal: with_override_hint(Diag::refused("gate refused"), message),
    }
}

/// `refusal` with the config's `message` as its first hint, when there is one,
/// and the way past the gate as the last.
fn with_override_hint(refusal: Diag, message: Option<&str>) -> Diag {
    match message {
        Some(message) => refusal.project_hint(message).hint(Hint::new(
            "or pass --force to skip the gate; the status note records it",
        )),
        None => refusal.hint(Hint::new(
            "pass --force to skip the gate; the status note records it",
        )),
    }
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
/// A waived gate that doesn't leave a trace is indistinguishable afterwards
/// from a gate that passed, which would make the whole mechanism advisory. The
/// note is where a status already explains itself. The waiver goes there,
/// appended to whatever `-m` supplied, or on its own when it supplied nothing.
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
#[path = "hooks_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "hooks_after_tests.rs"]
mod after_tests;

#[cfg(test)]
#[path = "hooks_approval_tests.rs"]
mod approval_tests;
