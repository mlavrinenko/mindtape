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

use std::collections::HashSet;
use std::io::IsTerminal;

use mindtape::config::hooks::{ActionKind, FlipAction, matching_rm_hooks};
use mindtape::shell::ConfigCommand;
use mindtape::{Filter, Readiness, Task, select};

use crate::diag::{Diag, Hint, Kind, Result};
use crate::render::RenderCtx;
use crate::render::output::{OutputFormat, render};
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
}

/// The `run` action's fields, grouped so the executor stays inside the
/// argument-count lint.
#[derive(Clone, Copy)]
struct Run<'a> {
    /// The command, or `None` for a run-less gate. `kind()` only ever pairs
    /// that with `gate: true`.
    command: Option<&'a ConfigCommand>,
    gate: bool,
    message: Option<&'a str>,
}

/// What a `run` action that fails without gating says when it doesn't declare
/// a `message`.
const DEFAULT_ADVICE: &str = "fix it, then run mt rm again";

/// Absolute path of the task the removal targets.
const ENV_FILE: &str = "MINDTAPE_TASK_FILE";
/// Identity of the task the removal targets (its file stem, unless `id:`
/// overrides).
const ENV_STEM: &str = "MINDTAPE_TASK_STEM";

impl<'a> Hooks<'a> {
    /// Build the runner for one removal: the project, the ids of the tasks it
    /// removes, and the readiness their `when` filters read.
    pub(super) fn new(
        project: &'a Project,
        eligible: &'a HashSet<String>,
        readiness: &'a Readiness,
    ) -> Self {
        Self {
            project,
            eligible,
            readiness,
        }
    }

    /// Run every action of every hook matching a removed task.
    ///
    /// A `show` or `list` action prints only when stdout is a terminal. It is
    /// presentation, and a scripted removal prints neither. A gate runs either
    /// way.
    ///
    /// # Errors
    /// Returns an error when a `when` or `list` expression fails to evaluate,
    /// an action doesn't declare a single kind, a command fails to spawn, or a
    /// gate refuses the removal.
    pub(super) fn run(&self) -> Result<()> {
        if self.project.config.on.rm.is_empty() {
            return Ok(());
        }
        let show = std::io::stdout().is_terminal();
        let mut allowed = None;
        for task in &self.project.outcome.tasks {
            if !self.eligible.contains(&task.id) {
                continue;
            }
            for hook in matching_rm_hooks(&self.project.config, task, self.readiness)? {
                for action in &hook.actions {
                    self.act(task, action, show, &mut allowed)?;
                }
            }
        }
        Ok(())
    }

    /// Run one action. A gate that fails returns the refusal, which stops the
    /// walk and the removal.
    fn act(
        &self,
        task: &Task,
        action: &FlipAction,
        show: bool,
        allowed: &mut Option<bool>,
    ) -> Result<()> {
        match action
            .kind()
            .map_err(|message| Diag::verbatim(Kind::Failed, format!("on.rm action: {message}")))?
        {
            ActionKind::Show(text) => {
                if show {
                    println!("{text}");
                }
                Ok(())
            }
            ActionKind::List { filter, title } => self.list(filter, title, show),
            ActionKind::Run {
                command,
                gate,
                message,
            } => self.exec(
                task,
                Run {
                    command,
                    gate,
                    message,
                },
                show,
                allowed,
            ),
        }
    }

    /// Print the tasks matching `spec` (a filter expression or `@view`) as a
    /// table, under `title`. A bare `@view` also picks up that view's sort.
    fn list(&self, spec: &str, title: Option<&str>, show: bool) -> Result<()> {
        if !show {
            return Ok(());
        }
        let project = self.project;
        let config = &project.config;
        let mut tasks = select(
            project.outcome.tasks.clone(),
            Some(&Filter::parse(spec)?),
            config,
            self.readiness,
        )?;
        let sort = spec
            .strip_prefix('@')
            .and_then(|name| config.views.get(name))
            .map(|view| view.sort.clone())
            .unwrap_or_default();
        crate::cmd::ls::sort_tasks(&mut tasks, &sort, config, self.readiness)?;
        if let Some(title) = title {
            println!("{title}");
        }
        if tasks.is_empty() {
            println!("  (no matching tasks)");
            return Ok(());
        }
        let hashes = crate::util::stem_hashes(&project.outcome.tasks, config);
        let env = crate::cmd::ls::rules::build_env(project, "table");
        let ctx = RenderCtx {
            config,
            readiness: self.readiness,
            env: &env,
            hashes: &hashes,
            root: &project.root,
            hyperlinks: project.hyperlinks,
            project: None,
        };
        println!("{}", render(&tasks, OutputFormat::Table, ctx, None)?);
        Ok(())
    }

    /// Execute a `run` action and judge the result, or, with no command at all,
    /// refuse outright: a run-less gate has nothing to run, so `when` matching
    /// is the whole verdict.
    fn exec(
        &self,
        task: &Task,
        run: Run<'_>,
        show: bool,
        allowed: &mut Option<bool>,
    ) -> Result<()> {
        let Some(spec) = run.command else {
            return Err(refused(run.message));
        };
        let command = spec.to_string();
        let allowed = *allowed.get_or_insert_with(|| {
            let config = &self.project.config;
            mindtape::allow::is_allowed(&self.project.root, &mindtape::allow::commands(config))
        });
        if !allowed {
            let project = self.project;
            mindtape::allow::hold(&project.root, &mindtape::allow::commands(&project.config));
            println!("skipped: {command} (not allowed)");
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
            if !run.gate && show {
                write_output(&command, &text);
            }
            return Ok(());
        }
        // A failure always surfaces, quiet or not: it is either the evidence a
        // refusal rests on, or a command the project asked to be told about.
        write_output(&command, &text);
        if !run.gate {
            let advice = run.message.unwrap_or(DEFAULT_ADVICE);
            let verdict = output.code.map_or_else(
                || "terminated by signal".to_owned(),
                |code| format!("exit {code}"),
            );
            println!("warning: {command} ({verdict}) — {advice}");
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
fn write_output(command: &str, output: &str) {
    println!("$ {command}");
    for line in output.lines() {
        println!("  {line}");
    }
}
