//! `[[tasks.rules]]` holds project-declared assertions over every task.
//!
//! Every other constraint `Mindtape` enforces is one it knows by name: a
//! status legend, a numeric scale, a tag allow-list, a heading path. A rule
//! constrains no field of its own, because it references the field itself, in
//! one of two shapes:
//!
//! - `assert` is an expression in the filter language, evaluated true/false:
//!
//!   ```toml
//!   [[tasks.rules]]
//!   name = "start-title-with-verb"
//!   assert = 'in_dict("verbs", word(title, 1))'
//!   when = '!terminal'
//!   message = "start the title with an imperative verb"
//!   severity = "error"
//!   ```
//!
//! - `check` is a Typst function the project already has, for a structure the
//!   filter language cannot express (see [`crate::rules::typst`]):
//!
//!   ```toml
//!   [[tasks.rules]]
//!   name = "give-every-goal-verdict"
//!   when = "!terminal"
//!   check = { typst = "truth/lib.typ", function = "check-goals", field = "x-goals" }
//!   severity = "error"
//!   ```
//!
//! `name`, `when`, `message` and `severity` work the same for either shape.
//! [`RuleSpec`] takes exactly one of `assert`/`check`, and [`validate_rules`]
//! refuses a rule declaring both or neither at config load.
//!
//! The validation pass evaluates rules ([`check_all`], called from
//! [`crate::validate::validate`]) rather than `mt check` alone. Every write
//! command validates before it writes, so `mt add`, `flip`, `link`, `import`,
//! `recur` and `review` enforce a rule for free. A convention that only fails
//! hours later at check time is a convention nobody keeps.
//!
//! A breach reports at the task's marker line, prefixed by its severity:
//!
//! ```text
//! error: tasks/rewrite-loader.typ:3: rules.start-title-with-verb: start the title with an imperative verb
//! ```
//!
//! `error` (the default) fails the command, and on a write refuses it before
//! anything lands. `warn` prints and lets the command through, the level an
//! advisory rule declares. That distinction is the [`Severity`] every
//! config-declared rule carries, and it travels out of the pass on
//! [`crate::validate::Violation::severity`]. That field is the one place a
//! violation's fatality is a project's choice rather than `Mindtape`'s.
//!
//! The config load dry-runs an `assert` rule's expression once
//! ([`validate_rules`]), the way it does `required`, so a malformed one is one
//! config error rather than one violation per artifact. A `check` rule's Typst
//! module gets the same existence and export check, but needs the project root
//! to read it. See [`crate::rules::typst::validate_load`], run from
//! [`crate::config::layers`] rather than from here, against [`RulesRoot`]
//! rather than a `root` parameter threaded through every validation call
//! site.
//!
//! # Files
//!
//! - `eval.rs`: the engine, compiling each declared entry and asking it about every task.
//! - `sites.rs`: where each rule comes from, so a report about the rule opens it.
//! - `typst.rs`: `check = { typst, ... }` rules, calling a project's own Typst function.

pub(crate) mod sites;
pub mod typst;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::model::Task;
use crate::problem::Problem;
use crate::validate::severity::Severity;

/// The project root a loaded [`Config`] carries, resolved once by
/// `crate::config::layers::build_config`. [`Config::compiled_dicts`] resolves
/// `[dicts]` `path` entries at load for the same reason, rather than making
/// every downstream filter evaluation take a `root` parameter to re-read the
/// section with. A `[[tasks.rules]] check` entry reads it here to resolve
/// `typst`, both at load ([`crate::rules::typst::validate_load`]) and per task
/// ([`crate::rules::typst::CompiledCheck`]).
///
/// Empty for a [`Config`] built through [`crate::config::parse_config`] alone
/// (no project on disk, as in most unit tests). A `check` rule then resolves
/// `typst` against the process's current directory. That is the same "best
/// effort, no project to root against" reading
/// [`crate::config::dicts::compile_dicts_rootless`] already takes for the
/// `path` entries of a dict.
///
/// Never equal-compared. Two configs with the same declared content are the
/// same config regardless of which directory loaded them. That is the reading
/// [`crate::config::dicts::DictCache`]'s own `PartialEq` already takes for the
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
    /// `crate::config::layers::build_config`.
    pub(crate) fn set(&mut self, root: &Path) {
        self.0 = root.to_owned();
    }

    pub(crate) fn as_path(&self) -> &Path {
        &self.0
    }
}

/// Call a Typst function instead of evaluating an `assert` expression, with
/// `check = { typst, function, field }`.
// `crate::rules::typst` evaluates the call.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CheckSpec {
    /// Path to the Typst module exporting `function`, relative to the project
    /// root (the directory holding `.mindtape/`), or to the directory of the
    /// pack that declares the rule.
    pub typst: String,
    /// Name of the function `typst` exports that this rule calls.
    pub function: String,
    /// Field whose value the function receives (`none` when the task omits
    /// it). Absent means the function receives the whole task, in the shape
    /// `mt eval --json` prints. `markers` is the one reserved input: it yields
    /// the task body's metadata markers as an array of `(label, value)` dicts
    /// in document order. It yields an empty array rather than `none` when the
    /// body emits none. Every other name is a canonical field or an `x-<ns>`
    /// slot.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// The directory `typst` resolves from and the check evaluates in, when a
    /// pack declared the rule: the pack's own directory. `None` means the
    /// project root. Set at load, never read from config.
    #[serde(skip)]
    pub(crate) root: Option<PathBuf>,
}

/// One assertion or check that every selected task must satisfy. Set exactly one
/// of `assert` and `check`.
// `validate_rules` refuses both or neither at config load. A hand-built
// `RuleSpec` (a test, or `parse_config` alone) can still carry either
// combination, so `eval::CompiledRule::compile` checks it again.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RuleSpec {
    /// The name a report shows for the rule (`rules.<name>`). Optional: an
    /// unnamed rule reports by its position instead (`rules[0]`).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Filter expression that must evaluate true for every task the rule
    /// applies to. The rule sets exactly one of this and `check`.
    #[serde(default, rename = "assert")]
    #[schemars(extend("x-expression" = "filter"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertion: Option<String>,
    /// A Typst function to call instead of an `assert` expression. The rule
    /// sets exactly one of this and `assert`.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check: Option<CheckSpec>,
    /// Filter expression gating which tasks the rule applies to. Absent means
    /// every task.
    #[serde(default)]
    #[schemars(extend("x-expression" = "filter"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
    /// What an `assert` breach tells the author to do, in place of the
    /// default advice that quotes the `assert` expression. A `check` rule
    /// ignores it and prints each string its function returns.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Whether a breach fails the command or only reports. Default `error`.
    #[serde(default)]
    pub severity: Severity,
}

impl RuleSpec {
    /// The label this rule reports under is its `name` if it has one, else its
    /// position in `[[tasks.rules]]`.
    ///
    /// The index is the fallback rather than the `assert` expression, because a
    /// report has to be greppable back to the config entry that produced it.
    /// Two rules may legitimately share an expression while differing in
    /// `when`. The expression still reaches the reader, as the default message,
    /// where it says what to fix rather than which rule to find.
    #[must_use]
    pub fn label(&self, index: usize) -> String {
        match &self.name {
            Some(name) => format!("rules.{name}"),
            None => format!("rules[{index}]"),
        }
    }

    /// The text an `assert` breach shows the author is the rule's `message`,
    /// else `must satisfy` and the expression. A `check` rule never reads this.
    pub(crate) fn advice(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            let expr = self.assertion.as_deref().unwrap_or_default();
            format!("must satisfy `{expr}`")
        })
    }
}

/// Compilation and evaluation of a declared rule, split out to keep this file
/// under its line budget. See [`check_all`].
mod eval;
pub(crate) use eval::check_all;

/// Check every `[[tasks.rules]]` entry's shape and `assert`/`when` expressions
/// at config load, returning one message per unusable one, prefixed with its
/// dotted TOML path.
///
/// A rule declaring both `assert` and `check`, or neither, is one problem.
/// The check parses both filter expressions and dry-runs them against a probe
/// task. An unknown field, view or dictionary then fails here, where it is one
/// config error, instead of once per artifact at validation time. A `check`
/// rule's Typst module needs the project root to read, which this function
/// does not have. [`crate::rules::typst::validate_load`] covers it separately,
/// from [`crate::config::layers`], the one caller with root in hand.
pub(crate) fn validate_rules(config: &Config) -> Vec<Problem> {
    let probe = Task::default();
    let mut problems = Vec::new();
    for (index, spec) in config.tasks.rules.iter().enumerate() {
        let path = format!("tasks.rules[{index}]");
        match (&spec.assertion, &spec.check) {
            (Some(_), Some(_)) => {
                problems.push(Problem::new(format!(
                    "{path}: exactly one of `assert`/`check` is allowed, not both"
                )));
            }
            (None, None) => {
                problems.push(Problem::new(format!(
                    "{path}: exactly one of `assert`/`check` is required"
                )));
            }
            _ => {}
        }
        if let Some(assertion) = &spec.assertion
            && let Some(problem) = crate::filter::expr_problem_in_set(assertion, &probe, config)
        {
            problems.push(problem.under(&format!("{path}.assert")));
        }
        if let Some(when) = &spec.when
            && let Some(problem) = crate::filter::expr_problem_in_set(when, &probe, config)
        {
            problems.push(problem.under(&format!("{path}.when")));
        }
    }
    problems
}

/// Cross-check for `[[tasks.rules]]`, folded into [`Config::validate`]: no two
/// entries share a `name`. Mirrors
/// [`crate::body::headers::validate_header_rules`] /
/// [`crate::body::content::validate_content_rules`]. `RuleSpec` already had a
/// `name` before either of those did, but never a check that two entries
/// couldn't share one.
///
/// This only ever sees a same-FILE duplicate. [`crate::config::packs`]'s merge
/// collapses two entries sharing a name to one, in place, the moment they come
/// from different sources. A nearer layer or an extended pack replaces a
/// farther one's same-named rule. By the time this runs, over the fully merged
/// [`Config::tasks`], two surviving entries with the same name can only mean
/// one file declared both. That file is a layer's own array, or a pack's.
pub(crate) fn validate_rule_names(config: &Config) -> Vec<String> {
    crate::body::headers::duplicate_rule_names(
        config.tasks.rules.iter().map(|rule| rule.name.as_deref()),
        "tasks.rules",
    )
}

/// Every `[[tasks.rules]]` entry's report label (`rules.<name>` or
/// `rules[<index>]`), for `crate::validate::candidate::elsewhere_violations` to
/// recognize a rule breach among the violations it attributes across a batch.
///
/// A set-aware rule, one asserting `count(...)` or reading the `links:` graph
/// through `ready`/`blocked`/`blocks`/`topo`, can start or stop breaching on a
/// task the batch never touches. The cause is a sibling candidate that changed
/// the set. A rule with no such dependency can never see that diff (nothing
/// about an unrelated task's fields moved). Widening the check to every rule
/// label is free, and it avoids a per-rule declaration of which kind it is.
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
/// ([`crate::Filter::reads_set_in`], directly or through a view): a cap on
/// `count(...)`, or a test of `ready`, `blocked`, `blocks` or `topo`.
///
/// A task cannot fix such a breach alone, so a check of one task apart from
/// its backlog drops them ([`crate::reads_set_breach`]).
pub(crate) fn set_labels(config: &Config) -> HashSet<String> {
    let reads_set = |source: &Option<String>| {
        source
            .as_deref()
            .and_then(|expr| crate::Filter::parse(expr).ok())
            .is_some_and(|filter| filter.reads_set_in(config))
    };
    (config.tasks.rules.iter().enumerate())
        .filter(|(_, spec)| reads_set(&spec.assertion) || reads_set(&spec.when))
        .map(|(index, spec)| spec.label(index))
        .collect()
}

#[cfg(test)]
mod tests;
