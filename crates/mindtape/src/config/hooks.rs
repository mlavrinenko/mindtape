//! `[[on.flip]]` — actions bound to a task changing status.
//!
//! One user-extensible mechanism for "when a task changes status, do
//! something". Printing the open set on the way into work and running a
//! machine-checkable command before a task may close are the same idea seen
//! twice: both are actions on a flip. Shipping them as separate tables would
//! give Mindtape two mechanisms for one concept, so there is one table,
//! several action kinds, and the project decides what fires where.
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
//! before the write** (so `status == "wip"` names the status being left), plus
//! the `into("name")` predicate for the status being stamped. `into` is not a
//! filter function: it is rewritten here, per transition, into a filter-language
//! constant ([`desugar_into`]) before the expression is parsed — so the whole
//! filter language composes with it and the evaluator stays unaware of it, the
//! same trick `@view` sugar plays in [`crate::filter::views`].
//!
//! Only `mt flip` runs actions. `mt check`, `mt ls` and every other read command
//! run none, because `mt check` runs inside editors and commit hooks, where a
//! test suite firing would be a surprise. The one config command a read runs is
//! a `[[tasks.waits]]` checker ([`crate::waits`]), which must answer from local
//! state. [`crate::shell`] runs both and states the trust boundary.
//!
//! This module holds the shape and the selection (which hooks fire for a given
//! transition); running the actions is the CLI's job.

use serde::Deserialize;
use thiserror::Error;

use crate::config::Config;
use crate::filter::{Filter, FilterError};
use crate::graph::ready::Readiness;
use crate::model::Task;
use crate::problem::Problem;
use crate::suggest::Unknown;

/// A filter atom matching every task: `id` is a plain `String` on [`Task`] and
/// is never absent, so `has(id)` is a tautology by construction. Used as the
/// value an `into(...)` call rewrites to when the transition matches.
const ALWAYS: &str = "has(id)";

/// The complement of [`ALWAYS`]: matches no task. What an `into(...)` call
/// rewrites to when the transition does not match.
const NEVER: &str = "miss(id)";

/// Hooks keyed by the event that fires them.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct OnConfig {
    /// `[[on.flip]]` entries, in declaration order.
    pub flip: Vec<FlipHook>,
}

/// One `[[on.flip]]` entry: which transitions it fires on, and what it does.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct FlipHook {
    /// Optional name. A later entry with the same name, from a nearer layer
    /// or a pack, replaces an earlier one in place, and `disable` drops it by
    /// name. An unnamed entry always appends and is never replaced. Also the
    /// label `mt rules` and `mt config show --origin` print.
    pub name: Option<String>,
    /// Filter over the task being flipped, plus `into("status")` for the status
    /// being stamped. Empty fires on every flip.
    #[schemars(extend("x-expression" = "filter"))]
    pub when: String,
    /// Actions to run, in declaration order.
    pub actions: Vec<FlipAction>,
}

/// One action of an `[[on.flip]]` entry. Set exactly one of `show`, `list`
/// and `run`, or set `gate = true` with no `run`.
// The kinds share one flat TOML shape, so a mistyped or conflicting entry
// fails as a config error naming the problem, rather than as a parse error.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct FlipAction {
    /// Print a message with `{ show = "text" }`.
    pub show: Option<String>,
    /// Print the matching tasks with `{ list = "filter or @view" }`.
    #[schemars(extend("x-expression" = "filter"))]
    pub list: Option<String>,
    /// Run a command through the platform shell (`sh -c`, or `cmd /C` on Windows)
    /// with `{ run = "command" }`. A gating action may omit it, and then the gate
    /// refuses the flip whenever the entry's `when` matches.
    pub run: Option<String>,
    /// Heading printed above a `list`'s table.
    pub title: Option<String>,
    /// Explanation printed when `run` exits non-zero, or when a gate with no
    /// `run` refuses the flip.
    pub message: Option<String>,
    /// Whether a non-zero exit from `run` refuses the flip. With no `run`, the
    /// gate refuses every flip the entry's `when` matches. Applies to a `run`
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
    /// Execute `command`, or — when `command` is `None` — nothing: a gate
    /// declaring no `run` refuses unconditionally whenever its hook's `when`
    /// matches. `kind()` only ever produces a `None` command alongside
    /// `gate: true`.
    Run {
        /// The command line, run through the platform shell (`sh -c`; `cmd /C`
        /// on Windows). `None` for a run-less gate.
        command: Option<&'a str>,
        /// Whether a non-zero exit (or, with no command, always) refuses the
        /// flip.
        gate: bool,
        /// Explanation printed on a non-zero exit or an unconditional refusal.
        message: Option<&'a str>,
    },
}

impl FlipAction {
    /// Resolve which kind this action declares.
    ///
    /// # Errors
    /// Returns a message naming the problem when the action declares no kind
    /// (no `show` / `list` / `run`, and no `gate = true` either), more than
    /// one, or a modifier belonging to a different kind (`title` outside
    /// `list`, `gate` / `message` outside `run`).
    pub fn kind(&self) -> Result<ActionKind<'_>, String> {
        match (
            self.show.as_deref(),
            self.list.as_deref(),
            self.run.as_deref(),
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

/// A `[[on.flip]]` entry whose `when` expression could not be evaluated.
#[derive(Debug, Error)]
#[error("on.flip[{index}].when: {source}: {}", crate::config::quote(.when))]
pub struct HookError {
    /// Position of the offending hook in `[[on.flip]]`.
    pub index: usize,
    /// The `when` expression that failed.
    pub when: String,
    /// The underlying filter failure.
    #[source]
    pub source: FilterError,
}

/// The `[[on.flip]]` hooks that fire when `task` is stamped `target`, in
/// declaration order.
///
/// `task` is the artifact as it stands before the write, so `when` reads the
/// status being left; `target` answers `into(...)`. `readiness` must be
/// computed over the whole loaded set (see [`Readiness::compute`]) for the
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
    let mut fired = Vec::new();
    for (index, hook) in config.on.flip.iter().enumerate() {
        let matched = fires(hook, task, target, config, readiness).map_err(|source| HookError {
            index,
            when: hook.when.clone(),
            source,
        })?;
        if matched {
            fired.push(hook);
        }
    }
    Ok(fired)
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

/// Rewrite every `into("name")` call in `when` into a filter-language constant:
/// [`ALWAYS`] when `name` is the status being stamped, [`NEVER`] otherwise. An
/// `into` inside a string literal, or one whose call shape does not parse, is
/// left verbatim — the filter parser then reports it as an unknown function.
#[must_use]
pub fn desugar_into(when: &str, target: &str) -> String {
    rewrite_into(when, |name| {
        if name == target { ALWAYS } else { NEVER }.to_owned()
    })
}

/// Rewrite every `into("name")` call in `when` into whatever `replace` makes
/// of its name — the one scanner behind both [`desugar_into`] (a flip's
/// constant) and a task-scope commit rule's state comparison
/// ([`crate::commit::task_expr::desugar_commit_into`]). String literals pass
/// through untouched, and `into` is recognised only as a whole identifier.
#[must_use]
pub fn rewrite_into(when: &str, replace: impl Fn(&str) -> String) -> String {
    scan_into(when, &replace).rewritten
}

/// Every status named by an `into(...)` call in `when`, in source order (may
/// contain duplicates). Used to check a hook against the configured legend at
/// load, so a typo'd status is one config error rather than a hook that quietly
/// never fires.
#[must_use]
pub fn into_targets(when: &str) -> Vec<String> {
    scan_into(when, &|_: &str| String::new()).targets
}

/// The result of one pass over a `when` expression.
struct IntoScan {
    /// `when` with every `into(...)` call replaced.
    rewritten: String,
    /// The status names those calls referenced.
    targets: Vec<String>,
}

/// Single pass over `when`: rewrite `into(...)` calls through `replace` and
/// collect the names they reference. String literals are copied through
/// untouched, and `into` is only recognised as a whole identifier — `zinto("x")`
/// and `into_view("x")` pass through.
fn scan_into(when: &str, replace: &dyn Fn(&str) -> String) -> IntoScan {
    let mut scan = IntoScan {
        rewritten: String::with_capacity(when.len()),
        targets: Vec::new(),
    };
    let mut cursor = 0;
    let mut in_string = false;
    let mut after_ident = false;
    // A `for` over a fixed bound, not `while cursor < when.len()`: every mutant
    // of the cursor advance below (`+=` -> `-=`/`*=`, a `copy_in_string` that
    // returns 0, a `parse_into_call` that reports length 0) otherwise spins
    // forever, and each turn pushes another char onto `rewritten` — measured
    // on 2026-09-29 at 45 GB RSS before the host's earlyoom killed the whole
    // sweep. Correct code consumes at least one byte per turn, so the bound is
    // never reached; a mutant now just returns a wrong scan.
    for _ in 0..when.len().saturating_add(1) {
        let Some(rest) = when.get(cursor..) else {
            break;
        };
        let Some(ch) = rest.chars().next() else { break };
        if in_string {
            cursor += copy_in_string(rest, ch, &mut scan.rewritten, &mut in_string);
            continue;
        }
        if !after_ident && let Some((name, len)) = parse_into_call(rest) {
            scan.rewritten.push_str(&replace(&name));
            scan.targets.push(name);
            cursor += len;
            after_ident = false;
            continue;
        }
        in_string = ch == '"';
        after_ident = is_ident_char(ch);
        scan.rewritten.push(ch);
        cursor += ch.len_utf8();
    }
    scan
}

/// Copy one character of a string literal through verbatim, consuming the
/// character after a backslash so an escaped quote does not close the literal.
/// Returns how many bytes were consumed and clears `in_string` on the closing
/// quote.
fn copy_in_string(rest: &str, ch: char, out: &mut String, in_string: &mut bool) -> usize {
    out.push(ch);
    let width = ch.len_utf8();
    if ch == '\\'
        && let Some(escaped) = rest.get(width..).and_then(|tail| tail.chars().next())
    {
        out.push(escaped);
        return width + escaped.len_utf8();
    }
    if ch == '"' {
        *in_string = false;
    }
    width
}

/// Parse an `into("name")` call at the start of `rest`, returning the name and
/// how many bytes the call occupies. `None` when `rest` does not open one.
fn parse_into_call(rest: &str) -> Option<(String, usize)> {
    let after_name = rest.strip_prefix("into")?;
    let open = after_name.trim_start().strip_prefix('(')?;
    let literal = open.trim_start().strip_prefix('"')?;
    let end = literal.find('"')?;
    let name = literal.get(..end)?.to_owned();
    let closed = literal.get(end + 1..)?.trim_start().strip_prefix(')')?;
    Some((name, rest.len() - closed.len()))
}

/// Whether `ch` can appear inside a filter-language identifier.
fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

/// Check every `[[on.flip]]` entry at config load, returning one message per
/// problem, prefixed with its dotted TOML path.
///
/// Each `when` is desugared against a status no legend can hold, parsed, and
/// dry-run against a probe task; each `into(...)` name is checked against the
/// legend; each action must resolve to exactly one kind, and a `list` filter
/// must parse. A malformed hook is one config error at load — not a surprise
/// halfway through a batch flip, with some tasks already written.
pub(crate) fn validate_flip_hooks(config: &Config) -> Vec<Problem> {
    let probe = Task::default();
    let mut problems = Vec::new();
    for (index, hook) in config.on.flip.iter().enumerate() {
        for name in into_targets(&hook.when) {
            let legend = &config.tasks.status;
            if !legend.is_empty() && !legend.contains(&name) {
                let known = legend.values();
                problems.push(
                    Problem::new(format!(
                        "on.flip[{index}].when: unknown status: {} in into()",
                        crate::config::quote(&name)
                    ))
                    .with(Unknown::near(&name, &known)),
                );
            }
        }
        if let Some(problem) =
            crate::filter::expr_problem_in_set(&desugar_into(&hook.when, ""), &probe, config)
        {
            problems.push(problem.under(&format!("on.flip[{index}].when")));
        }
        if hook.actions.is_empty() {
            problems.push(Problem::new(format!(
                "on.flip[{index}]: no actions declared"
            )));
        }
        problems.extend(action_problems(index, hook, &probe, config));
    }
    problems
}

/// Load-time problems in one hook's actions: an unresolvable kind, or a `list`
/// filter that does not parse or evaluate.
fn action_problems(index: usize, hook: &FlipHook, probe: &Task, config: &Config) -> Vec<Problem> {
    let mut problems = Vec::new();
    for (slot, action) in hook.actions.iter().enumerate() {
        let at = format!("on.flip[{index}].actions[{slot}]");
        match action.kind() {
            Err(message) => problems.push(Problem::new(format!("{at}: {message}"))),
            Ok(ActionKind::List { filter, .. }) => {
                if let Some(problem) = crate::filter::expr_problem_in_set(filter, probe, config) {
                    problems.push(problem.under(&format!("{at}.list")));
                }
            }
            Ok(_) => {}
        }
    }
    problems
}

#[cfg(test)]
#[path = "hooks_tests.rs"]
mod tests;
