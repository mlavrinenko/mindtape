//! The task half of a `scope = "task"` commit rule's expression: reading one
//! task as a commit state holds it.
//!
//! A task-scope rule runs once per task the commit changes or names
//! ([`crate::commit_msg::CommitRecord::tasks`]). It reads the commit's own
//! fields as a commit rule does ([`crate::commit_expr`]), plus:
//!
//! - `head(<expr>)`, `staged(<expr>)`, `worktree(<expr>)` — any filter
//!   expression, graph predicates and `count(...)` included, evaluated over
//!   the task as that state holds it, against that state's whole task set. As
//!   a comparison side, `staged(status)` is the field's value in that state.
//! - `exists`, inside a wrapper — whether the state holds the task's file.
//! - `named` — the commit's trailer names the task.
//! - `into("s")` — `staged(status == "s") && !head(status == "s")`, rewritten
//!   before the parse by the scanner `[[on.flip]]` shares
//!   ([`crate::config_hooks::rewrite_into`]).
//!
//! A bare task field is refused: which state it would read is the one thing
//! a reader of the rule could not tell.

use evalexpr::{DefaultNumericTypes, Node};

use crate::commit_expr::CommitExpr;
use crate::commit_msg::CommitRecord;
use crate::commit_tasks::{CommitStates, ProjectState};
use crate::config::Config;
use crate::expr::call_args;
use crate::fields::FieldValue;
use crate::filter::{Ctx, FilterError, eval_bool};
use crate::model::Task;

/// The state wrappers, in the order an error lists them.
pub(crate) const STATES: [&str; 3] = ["head", "staged", "worktree"];

/// The task a task-scope rule is evaluating, and the states it reads.
#[derive(Debug, Clone, Copy)]
pub struct TaskScope<'a> {
    /// The task's stem.
    pub stem: &'a str,
    /// The project in every state the commit is checked against.
    pub states: &'a CommitStates,
    /// The config the inner filter expressions resolve against.
    pub config: &'a Config,
    /// The load-time dry run: `changed()` checks its arguments, reads no git.
    pub(crate) dry: bool,
}

impl<'a> TaskScope<'a> {
    /// The scope of the task `stem` over `states`.
    #[must_use]
    pub const fn new(stem: &'a str, states: &'a CommitStates, config: &'a Config) -> Self {
        Self {
            stem,
            states,
            config,
            dry: false,
        }
    }

    /// `named`: the trailer names this task.
    pub(crate) fn named(&self, commit: &CommitRecord) -> bool {
        commit.refs.iter().any(|stem| stem == self.stem)
    }

    fn state(&self, name: &str) -> Result<&'a ProjectState, FilterError> {
        match name {
            "head" => Ok(&self.states.head),
            "staged" => Ok(&self.states.staged),
            _ => self.states.worktree.as_ref().ok_or_else(|| {
                FilterError::Invalid("worktree(): there is no working tree to read".to_owned())
            }),
        }
    }

    /// Evaluate `read` over the task as state `name` holds it — a stand-in
    /// carrying only the stem where the state lacks the file.
    fn read<T>(
        &self,
        name: &str,
        children: &[Node<DefaultNumericTypes>],
        read: impl Fn(&Node<DefaultNumericTypes>, &Ctx) -> Result<T, FilterError>,
    ) -> Result<T, FilterError> {
        let [inner] = call_args(children)[..] else {
            return Err(FilterError::Invalid(format!(
                "{name}() takes one expression: {name}(status == \"done\")"
            )));
        };
        let state = self.state(name)?;
        let absent = Task {
            id: self.stem.to_owned(),
            ..Task::default()
        };
        let found = state.task(self.stem);
        let task = found.unwrap_or(&absent);
        let ctx = Ctx::in_state(
            task,
            self.config,
            &state.readiness,
            found.is_some(),
            self.dry,
        );
        read(inner, &ctx)
    }

    /// `head(<expr>)` and its siblings in boolean position.
    pub(crate) fn holds(
        &self,
        name: &str,
        children: &[Node<DefaultNumericTypes>],
    ) -> Result<bool, FilterError> {
        self.read(name, children, eval_bool)
    }

    /// `staged(<field>)` and its siblings as a comparison side.
    pub(crate) fn value(
        &self,
        name: &str,
        children: &[Node<DefaultNumericTypes>],
    ) -> Result<FieldValue, FilterError> {
        self.read(name, children, crate::filter_compare::left_operand)
    }
}

/// Rewrite every `into("s")` in a task-scope expression into the flip it
/// means: `staged(status == "s") && !head(status == "s")`.
#[must_use]
pub fn desugar_commit_into(expr: &str) -> String {
    crate::config_hooks::rewrite_into(expr, |name| {
        format!("(staged(status == \"{name}\") && !head(status == \"{name}\"))")
    })
}

/// What makes one task-scope expression unusable, if anything: an `into(...)`
/// naming no configured status, a syntax error, or what a dry run over a
/// probe task present in every state surfaces — a bare task field above all.
#[must_use]
pub(crate) fn task_expr_problem(source: &str, config: &Config) -> Option<String> {
    let legend = &config.tasks.status;
    if let Some(name) = crate::config_hooks::into_targets(source)
        .into_iter()
        .find(|name| !legend.is_empty() && !legend.contains(name))
    {
        return Some(format!("into({name:?}) names no configured status"));
    }
    let probe = ProjectState::probe(config);
    let states = CommitStates {
        head: probe.clone(),
        staged: probe.clone(),
        worktree: Some(probe),
    };
    let scope = TaskScope {
        dry: true,
        ..TaskScope::new("", &states, config)
    };
    CommitExpr::parse(&desugar_commit_into(source))
        .and_then(|expr| expr.matches_task(&CommitRecord::default(), &scope))
        .err()
        .map(|err| err.to_string())
}

#[cfg(test)]
#[path = "commit_task_expr_tests.rs"]
mod tests;
