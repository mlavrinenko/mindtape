//! `[[tasks.rules]]` — project-declared assertions over every task.
//!
//! Every other constraint `MindTape` enforces is one it knows by name: a
//! status legend, a numeric scale, a tag allow-list, a heading path. A rule
//! constrains no field in particular, because it names the field itself, in
//! one of two shapes:
//!
//! - `assert` — an expression in the filter language, evaluated true/false:
//!
//!   ```toml
//!   [[tasks.rules]]
//!   name = "imperative-title"
//!   assert = 'in_dict("verbs", word(title, 1))'
//!   when = '!terminal'
//!   message = "start the title with an imperative verb"
//!   severity = "error"
//!   ```
//!
//! - `check` — a Typst function the project already has, for a structure no
//!   filter expression can name (see [`crate::config_rules_typst`]):
//!
//!   ```toml
//!   [[tasks.rules]]
//!   name = "goals"
//!   when = "!terminal"
//!   check = { typst = "truth/lib.typ", function = "check-goals", field = "x-goals" }
//!   severity = "error"
//!   ```
//!
//! `name`, `when`, `message` and `severity` work the same for either shape.
//! [`RuleSpec`] takes exactly one of `assert`/`check` — [`validate_rules`]
//! refuses a rule declaring both or neither at config load.
//!
//! Rules are evaluated inside the validation pass ([`check_all`], called from
//! [`crate::validate::validate`]) rather than in `mt check` alone. Every write
//! command validates before it writes, so `mt add`, `flip`, `link`, `import`,
//! `recur` and `review` enforce a rule for free — a convention that only fails
//! hours later at check time is a convention nobody keeps.
//!
//! A breach reports at the task's marker line, prefixed by its severity:
//!
//! ```text
//! error: tasks/rewrite-loader.typ:3: rules.imperative-title: start the title with an imperative verb
//! ```
//!
//! `warn` (the default) prints and lets the command through; `error` fails it,
//! and on a write refuses it before anything lands. That distinction is the
//! [`Severity`] `[[tasks.limits]]` already uses, and it travels out of the pass
//! on [`Violation::severity`] — the one place a violation's fatality is a
//! project's choice rather than `MindTape`'s.
//!
//! An `assert` rule's expression is dry-run once at config load
//! ([`validate_rules`]), the way `required` and `[[tasks.limits]]` are, so a
//! malformed one is one config error rather than one violation per artifact. A
//! `check` rule's Typst module is checked for existence and export the same
//! way, but needs the project root to read it — see
//! [`crate::config_rules_typst::validate_load`], run from
//! [`crate::config_layers`] rather than from here, against [`RulesRoot`]
//! rather than a `root` parameter threaded through every validation call
//! site.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::config::Config;
use crate::model::Task;
use crate::severity::Severity;

/// The project root a loaded [`Config`] was built for, resolved once by
/// [`crate::config_layers::build_config`] and carried on the config itself —
/// the same reason [`Config::compiled_dicts`] resolves `[dicts]` `path`
/// entries at load rather than making every downstream filter evaluation
/// take a `root` parameter to re-read the section with. A `[[tasks.rules]]
/// check` entry reads it here to resolve `typst`, both at load
/// ([`crate::config_rules_typst::validate_load`]) and per task
/// ([`crate::config_rules_typst::CompiledCheck`]).
///
/// Empty for a [`Config`] built through [`crate::config::parse_config`] alone
/// (no project on disk, e.g. most unit tests) — a `check` rule then resolves
/// `typst` against the process's current directory, the same "best effort,
/// no project to root against" reading [`crate::config_dicts::compile_dicts_rootless`]
/// already takes for a dict's own `path` entries.
///
/// Never equal-compared: two configs with the same declared content are the
/// same config regardless of which directory loaded them, the reading
/// [`crate::config_dicts::DictCache`]'s own `PartialEq` already takes for the
/// same kind of load-derived state.
#[derive(Debug, Clone, Default)]
pub(crate) struct RulesRoot(PathBuf);

impl PartialEq for RulesRoot {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl RulesRoot {
    /// Set the resolved project root. Called once, from
    /// [`crate::config_layers::build_config`].
    pub(crate) fn set(&mut self, root: &Path) {
        self.0 = root.to_owned();
    }

    fn as_path(&self) -> &Path {
        &self.0
    }
}

/// `check = { typst, function, field }` — a `[[tasks.rules]]` entry that calls
/// a Typst function instead of evaluating an `assert` expression.
// `crate::config_rules_typst` evaluates the call.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CheckSpec {
    /// Path to the Typst module exporting `function`, relative to the project
    /// root (the directory holding `.mindtape/`).
    pub typst: String,
    /// Name of the function `typst` exports that this rule calls.
    pub function: String,
    /// Field whose value the function receives (`none` when the task omits
    /// it). Absent means the function receives the whole task, in the shape
    /// `mt eval --json` prints.
    #[serde(default)]
    pub field: Option<String>,
}

/// `[[tasks.rules]]` — one assertion or check every selected task must
/// satisfy. Set exactly one of `assert` and `check`.
// `validate_rules` refuses both or neither at config load. A hand-built
// `RuleSpec` (a test, or `parse_config` alone) can still carry either
// combination, so `eval::CompiledRule::compile` checks it again.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RuleSpec {
    /// The name a report shows for the rule (`rules.<name>`). Optional: an
    /// unnamed rule reports by its position instead (`rules[0]`).
    #[serde(default)]
    pub name: Option<String>,
    /// Filter expression that must evaluate true for every task the rule
    /// applies to. Exactly one of this and `check` is set.
    #[serde(default, rename = "assert")]
    pub assertion: Option<String>,
    /// A Typst function to call instead of an `assert` expression. Exactly
    /// one of this and `assert` is set.
    #[serde(default)]
    pub check: Option<CheckSpec>,
    /// Filter expression gating which tasks the rule applies to. Absent means
    /// every task.
    #[serde(default)]
    pub when: Option<String>,
    /// What an `assert` breach tells the author to do, in place of the
    /// default advice naming the `assert` expression. A `check` rule ignores
    /// it and prints each string its function returns.
    #[serde(default)]
    pub message: Option<String>,
    /// Whether a breach fails the command or only reports. Default `warn`.
    #[serde(default)]
    pub severity: Severity,
}

impl RuleSpec {
    /// How this rule identifies itself in a report: its `name` if it has one,
    /// else its position in `[[tasks.rules]]`.
    ///
    /// The index is the fallback rather than the `assert` expression because a
    /// report has to be greppable back to the config entry that produced it,
    /// and two rules may legitimately share an expression while differing in
    /// `when`. The expression still reaches the reader — as the default
    /// message, where it says what to fix rather than which rule to find.
    #[must_use]
    pub fn label(&self, index: usize) -> String {
        match &self.name {
            Some(name) => format!("rules.{name}"),
            None => format!("rules[{index}]"),
        }
    }

    /// What an `assert` breach tells the author. Only called on the `assert`
    /// arm of [`eval::CompiledRule::compile`] — a `check` rule never reads this.
    fn advice(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            let expr = self.assertion.as_deref().unwrap_or_default();
            format!("must satisfy `{expr}`")
        })
    }
}

/// Compilation and evaluation of a declared rule; split out to keep this file
/// under its line budget. See [`check_all`].
#[path = "config_rules_eval.rs"]
mod eval;
pub(crate) use eval::check_all;

/// Check every `[[tasks.rules]]` entry's shape and `assert`/`when` expressions
/// at config load, returning one message per unusable one, prefixed with its
/// dotted TOML path.
///
/// A rule declaring both `assert` and `check`, or neither, is one problem.
/// Both filter expressions are parsed and dry-run against a probe task, so an
/// unknown field, view or dictionary is caught here — where it is one config
/// error — instead of once per artifact at validation time. A `check` rule's
/// Typst module needs the project root to read, which this function does not
/// have; [`crate::config_rules_typst::validate_load`] covers it separately,
/// from [`crate::config_layers`], the one caller with root in hand.
pub(crate) fn validate_rules(config: &Config) -> Vec<String> {
    let probe = Task::default();
    let mut problems = Vec::new();
    for (index, spec) in config.tasks.rules.iter().enumerate() {
        let path = format!("tasks.rules[{index}]");
        match (&spec.assertion, &spec.check) {
            (Some(_), Some(_)) => {
                problems.push(format!(
                    "{path}: exactly one of `assert`/`check` is allowed, not both"
                ));
            }
            (None, None) => {
                problems.push(format!(
                    "{path}: exactly one of `assert`/`check` is required"
                ));
            }
            _ => {}
        }
        if let Some(assertion) = &spec.assertion
            && let Some(problem) = crate::filter::expr_problem_in_set(assertion, &probe, config)
        {
            problems.push(format!("{path}.assert: {problem}"));
        }
        if let Some(when) = &spec.when
            && let Some(problem) = crate::filter::expr_problem_in_set(when, &probe, config)
        {
            problems.push(format!("{path}.when: {problem}"));
        }
    }
    problems
}

/// Cross-check for `[[tasks.rules]]`, folded into [`Config::validate`]: no two
/// entries share a `name`. Mirrors
/// [`crate::body_headers::validate_header_rules`] /
/// [`crate::body_content::validate_content_rules`] — `RuleSpec` already had a
/// `name` before either of those did, but never a check that two entries
/// couldn't share one.
///
/// This only ever sees a same-FILE duplicate: [`crate::config_packs`]'s merge
/// collapses two entries sharing a name to one, in place, the moment they
/// come from different sources (a nearer layer or an extended pack replacing
/// a farther one's same-named rule), so by the time this runs — over the
/// fully merged [`Config::tasks`] — two surviving entries with the same name
/// can only mean one file (a layer's own array, or a pack's) declared both.
pub(crate) fn validate_rule_names(config: &Config) -> Vec<String> {
    crate::body_headers::duplicate_rule_names(
        config.tasks.rules.iter().map(|rule| rule.name.as_deref()),
        "tasks.rules",
    )
}

/// Every `[[tasks.rules]]` entry's report label (`rules.<name>` or
/// `rules[<index>]`), for [`crate::candidate::elsewhere_violations`] to
/// recognize a rule breach among the violations it attributes across a batch.
///
/// A set-aware rule — one asserting `count(...)`, or reading the `links:`
/// graph through `ready`/`blocked`/`blocks`/`topo` — can start or stop
/// breaching on a task the batch never touches, purely because a sibling
/// candidate changed the set. A rule with no such dependency can never see
/// that diff (nothing about an unrelated task's fields moved), so widening
/// the check to every rule label costs nothing and needs no per-rule
/// declaration of which kind it is.
pub(crate) fn labels(config: &Config) -> HashSet<String> {
    config
        .tasks
        .rules
        .iter()
        .enumerate()
        .map(|(index, spec)| spec.label(index))
        .collect()
}

/// The labels of the rules whose `assert` or `when` reads the whole task set
/// ([`crate::Filter::reads_set`]): a cap on `count(...)`, or a test of
/// `ready`, `blocked`, `blocks` or `topo`.
///
/// A task cannot fix such a breach alone, so a check of one task apart from
/// its backlog drops them ([`crate::reads_set_breach`]).
pub(crate) fn set_labels(config: &Config) -> HashSet<String> {
    let reads_set = |source: &Option<String>| {
        source
            .as_deref()
            .and_then(|expr| crate::Filter::parse(expr).ok())
            .is_some_and(|filter| filter.reads_set())
    };
    (config.tasks.rules.iter().enumerate())
        .filter(|(_, spec)| reads_set(&spec.assertion) || reads_set(&spec.when))
        .map(|(index, spec)| spec.label(index))
        .collect()
}

#[cfg(test)]
#[path = "config_rules_tests.rs"]
mod tests;
