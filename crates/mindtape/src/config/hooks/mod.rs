//! `[[on.flip]]` binds actions to a task changing status, and `[[on.rm]]` to a
//! task's removal.
//!
//! One user-extensible mechanism for "when a task changes, do something".
//! Printing the open set on the way into work and running a machine-checkable
//! command before a task may close are the same idea seen twice. Both are
//! actions on a flip. Two separate tables would give `Mindtape` two mechanisms
//! for one concept, so there is one table, a few action kinds, and the config
//! states what fires where.
//!
//! ```toml
//! [[on.flip]]
//! when = 'into("wip")'
//! actions = [{ list = "@wip", title = "Already in flight" }]
//!
//! [[on.flip]]
//! name = "tests-pass"
//! when = 'into("done") && !has_tag("chore")'
//! actions = [{ run = "just check", gate = true, message = "tests must pass" }]
//! ```
//!
//! `[[on.flip]]` entries append across packs and layers, a later entry named
//! like an earlier one replaces it in place, and `disable` drops one by name
//! ([`crate::config::packs`]), as for `[[tasks.rules]]`.
//!
//! `when` is a [filter expression](crate::filter) over the task **as it stands
//! before the write** (so `status == "wip"` names the status the task leaves),
//! plus the `into("name")` predicate for the status the flip stamps. `into` is
//! not a filter function: the scanner in `into.rs` rewrites it, per transition,
//! into a filter-language constant ([`desugar_into`]) before the expression
//! parses. The whole filter language then composes with it and the evaluator
//! stays unaware of it, the same trick `@view` sugar plays in
//! [`crate::filter::views`].
//!
//! Only `mt flip` and `mt rm` run actions. `mt check`, `mt ls` and every other
//! read command run none, because `mt check` runs inside editors and commit
//! hooks, where a test suite firing would be a surprise. The one config command
//! a read runs is a `[[tasks.waits]]` checker ([`crate::waits`]), which must
//! answer from local state. [`crate::shell`] runs both and states the trust
//! boundary.
//!
//! This module holds the shape and the selection (which hooks fire for a given
//! transition). Running the actions is the CLI's job.
//!
//! # Files
//!
//! - `into.rs`: the `into("status")` scanner a flip's `when` is rewritten with.
//! - `rm.rs`: `[[on.rm]]`, actions bound to a removal.
//! - `validate.rs`: the load-time checks both hook arrays share.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::Config;
use crate::config::add_hooks::AddHook;
use crate::filter::{Filter, FilterError};
use crate::graph::ready::Readiness;
use crate::model::Task;
use crate::shell::ConfigCommand;

/// Hooks keyed by the event that fires them.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct OnConfig {
    /// `[[on.flip]]` entries, in declaration order.
    pub flip: Vec<FlipHook>,
    /// `[[on.add]]` entries, in declaration order. Their shape and validation
    /// live in the add-hook config module.
    pub add: Vec<AddHook>,
    /// `[[on.rm]]` entries, in declaration order. Their shape and validation
    /// live in the rm-hook config module.
    pub rm: Vec<RmHook>,
}

/// One `[[on.flip]]` entry: which transitions it fires on, and what it does.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct FlipHook {
    /// Optional name. A later entry with the same name, from a nearer layer
    /// or a pack, replaces an earlier one in place, and `disable` drops it by
    /// name. An unnamed entry always appends and is never replaced. Also the
    /// label `mt explain rules` and `mt config show --origin` print.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Filter over the task the flip acts on, plus `into("status")` for the
    /// status the flip stamps. Empty fires on every flip.
    #[schemars(extend("x-expression" = "filter"))]
    pub when: String,
    /// Actions to run, in declaration order.
    pub actions: Vec<FlipAction>,
}

/// The fields every `[[on.*]]` hook carries, so one selection, one listing and
/// one load-time check read both hook arrays.
pub(crate) trait HookFields {
    /// The hook's `name`, `when` and `actions`, in that order.
    fn parts(&self) -> (Option<&str>, &str, &[FlipAction]);
}

/// Implement [`HookFields`] for a hook struct whose fields are `name`, `when`
/// and `actions`, the shape every `[[on.*]]` entry shares.
macro_rules! impl_hook_fields {
    ($hook:ty) => {
        impl HookFields for $hook {
            fn parts(&self) -> (Option<&str>, &str, &[FlipAction]) {
                (self.name.as_deref(), &self.when, &self.actions)
            }
        }
    };
}
pub(crate) use impl_hook_fields;

impl_hook_fields!(FlipHook);

/// One action of an `[[on.flip]]` or `[[on.rm]]` entry. Set exactly one of
/// `show`, `list` and `run`, or set `gate = true` with no `run`.
// The kinds share one flat TOML shape, so a mistyped or conflicting entry
// fails as a config error naming the problem, rather than as a parse error.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct FlipAction {
    /// Print a message with `{ show = "text" }`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show: Option<String>,
    /// Print the matching tasks with `{ list = "filter or @view" }`.
    #[schemars(extend("x-expression" = "filter"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<String>,
    /// Run a command with `{ run = "command" }`, through the platform shell
    /// (`sh -c`, or `cmd /C` on Windows), or with `{ run = ["program", "arg"] }`,
    /// directly with no shell. A gating action may omit it, and then the gate
    /// refuses the write whenever the entry's `when` matches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<ConfigCommand>,
    /// Heading printed above a `list`'s table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Explanation printed when `run` exits non-zero, or when a gate with no
    /// `run` refuses the write.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Whether a non-zero exit from `run` refuses the write. With no `run`, the
    /// gate refuses every write the entry's `when` matches. Applies to a `run`
    /// action or a gate with no `run`.
    pub gate: bool,
}

/// A [`FlipAction`] resolved to the one kind it declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind<'a> {
    /// Print `.0` verbatim.
    Show(&'a str),
    /// Print the tasks matching `filter`, under an optional `title`.
    List {
        /// Filter expression or `@view` reference.
        filter: &'a str,
        /// Heading printed above the table.
        title: Option<&'a str>,
    },
    /// Execute `command`, or nothing when `command` is `None`: a gate
    /// declaring no `run` refuses unconditionally whenever its hook's `when`
    /// matches. `kind()` only ever produces a `None` command alongside
    /// `gate: true`.
    Run {
        /// The command, either a line for the platform shell or an array that
        /// runs without a shell. `None` for a run-less gate.
        command: Option<&'a ConfigCommand>,
        /// Whether a non-zero exit (or, with no command, always) refuses the
        /// write.
        gate: bool,
        /// Explanation printed on a non-zero exit or an unconditional refusal.
        message: Option<&'a str>,
    },
}

impl FlipAction {
    /// Resolve which kind this action declares.
    ///
    /// # Errors
    /// Returns a message that specifies the problem when the action has no
    /// kind (no `show` / `list` / `run`, and no `gate = true` either), more
    /// than one, or a modifier belonging to a different kind (`title` outside
    /// `list`, `gate` / `message` outside `run`).
    pub fn kind(&self) -> Result<ActionKind<'_>, String> {
        match (
            self.show.as_deref(),
            self.list.as_deref(),
            self.run.as_ref(),
        ) {
            (Some(text), None, None) => {
                reject(&[
                    ("title", self.title.is_some(), "list"),
                    ("gate", self.gate, "run"),
                    ("message", self.message.is_some(), "run"),
                ])?;
                Ok(ActionKind::Show(text))
            }
            (None, Some(filter), None) => {
                reject(&[
                    ("gate", self.gate, "run"),
                    ("message", self.message.is_some(), "run"),
                ])?;
                Ok(ActionKind::List {
                    filter,
                    title: self.title.as_deref(),
                })
            }
            // A command runs it; `gate = true` alone (no command) still
            // resolves — a run-less gate that refuses unconditionally.
            (None, None, command) if command.is_some() || self.gate => {
                reject(&[("title", self.title.is_some(), "list")])?;
                Ok(ActionKind::Run {
                    command,
                    gate: self.gate,
                    message: self.message.as_deref(),
                })
            }
            (None, None, None) => {
                Err("action declares no kind: set show, list, run or gate = true".to_owned())
            }
            _ => Err("action declares more than one of show, list and run".to_owned()),
        }
    }
}

/// Reject any modifier in `set` that is present but belongs to another action
/// kind, naming the kind that owns it.
fn reject(set: &[(&str, bool, &str)]) -> Result<(), String> {
    for (name, present, owner) in set {
        if *present {
            return Err(format!(
                "{} applies only to {owner} actions",
                crate::config::quote(name)
            ));
        }
    }
    Ok(())
}

/// A hook whose `when` expression failed to evaluate.
#[derive(Debug, Error)]
#[error("{section}[{index}].when: {source}: {}", crate::config::quote(.when))]
pub struct HookError {
    /// The hook array the entry sits in: `on.flip` or `on.rm`.
    pub section: &'static str,
    /// Position of the offending hook in that array.
    pub index: usize,
    /// The `when` expression that failed.
    pub when: String,
    /// The underlying filter failure. Boxed, so a `Result` that carries a
    /// `HookError` stays under clippy's `result_large_err` size.
    #[source]
    pub source: Box<FilterError>,
}

/// The `[[on.flip]]` hooks that fire when the flip stamps `task` `target`, in
/// declaration order.
///
/// `task` is the artifact as it stands before the write, so `when` reads the
/// status the task leaves. `target` answers `into(...)`. The caller computes
/// `readiness` over the whole loaded set (see [`Readiness::compute`]) for the
/// `ready` / `blocked` predicates.
///
/// # Errors
/// Returns [`HookError`] when a `when` expression fails to parse or names an
/// unknown field or view.
pub fn matching_flip_hooks<'a>(
    config: &'a Config,
    task: &Task,
    target: &str,
    readiness: &Readiness,
) -> Result<Vec<&'a FlipHook>, HookError> {
    matching(&config.on.flip, "on.flip", |hook| {
        fires(hook, task, target, config, readiness)
    })
}

/// The hooks of `hooks` whose `when` matches, in declaration order. `section`
/// is the array they sit in, for the error a bad `when` reports.
///
/// # Errors
/// Returns [`HookError`] when a `when` expression fails to parse or names an
/// unknown field or view.
pub(crate) fn matching<'a, H: HookFields>(
    hooks: &'a [H],
    section: &'static str,
    fires: impl Fn(&H) -> Result<bool, FilterError>,
) -> Result<Vec<&'a H>, HookError> {
    let mut hits = Vec::new();
    for (index, hook) in hooks.iter().enumerate() {
        let hit = fires(hook).map_err(|source| HookError {
            section,
            index,
            when: hook.parts().1.to_owned(),
            source: Box::new(source),
        })?;
        if hit {
            hits.push(hook);
        }
    }
    Ok(hits)
}

/// Whether one hook's `when` matches this transition.
fn fires(
    hook: &FlipHook,
    task: &Task,
    target: &str,
    config: &Config,
    readiness: &Readiness,
) -> Result<bool, FilterError> {
    Filter::parse(&desugar_into(&hook.when, target))?.matches_with(task, config, readiness)
}

mod into;
pub(crate) use into::{ALWAYS, NEVER, is_ident_char, parse_named_call, scan_expression};
pub use into::{desugar_into, into_targets, rewrite_into};

mod rm;
pub use rm::{RmHook, matching_rm_hooks};

mod validate;
pub(crate) use validate::{validate_flip_hooks, validate_rm_hooks};

#[cfg(test)]
mod rm_tests;

#[cfg(test)]
mod tests;
