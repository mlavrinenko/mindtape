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
use crate::config_rules_typst::CompiledCheck;
use crate::filter::{Filter, FilterError};
use crate::model::Task;
use crate::ready::Readiness;
use crate::severity::Severity;
use crate::validate::Violation;

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

/// `check = { typst, function, field }` — a `[[tasks.rules]]` entry that
/// delegates to a Typst function instead of a filter expression. See
/// [`crate::config_rules_typst`] for the evaluation route.
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
/// satisfy.
///
/// Exactly one of `assertion`/`check` is set on a rule that came through
/// [`crate::config::load_config`] — [`validate_rules`] refuses both or
/// neither at config load — but a hand-built `RuleSpec` (a test, or
/// `parse_config` alone) can carry either combination, so
/// [`CompiledRule::compile`] still treats it as a fact to check rather than an
/// invariant to assume.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RuleSpec {
    /// What the rule is called, used to report it (`rules.<name>`). Optional:
    /// an unnamed rule reports by its position instead (`rules[0]`).
    #[serde(default)]
    pub name: Option<String>,
    /// Filter expression that must evaluate true for every task the rule
    /// applies to. Exactly one of this and `check` is set.
    #[serde(default, rename = "assert")]
    pub assertion: Option<String>,
    /// A Typst function to call instead of an `assert` expression. Exactly
    /// one of this and `assertion` is set.
    #[serde(default)]
    pub check: Option<CheckSpec>,
    /// Filter expression gating which tasks the rule applies to. Absent means
    /// every task.
    #[serde(default)]
    pub when: Option<String>,
    /// What an `assert` breach tells the author to do, in place of the
    /// default advice naming the `assert` expression itself. Has no effect on
    /// a `check` rule: each string the function returns is already the
    /// specific thing to fix.
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
    /// arm of [`CompiledRule::compile`] — a `check` rule never reads this.
    fn advice(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            let expr = self.assertion.as_deref().unwrap_or_default();
            format!("must satisfy `{expr}`")
        })
    }
}

/// The compiled form of a rule's own assertion, once its shape is known.
enum RuleKind {
    /// `assert`: the parsed filter, plus the breach message to report on a
    /// `false` evaluation.
    Assert { filter: Filter, advice: String },
    /// `check`: the Typst function to call, reused across every task this
    /// rule is asked about.
    Check(CompiledCheck),
}

/// One `[[tasks.rules]]` entry with its shape compiled, ready to ask task by
/// task.
struct CompiledRule {
    label: String,
    when: Option<Filter>,
    kind: RuleKind,
    severity: Severity,
}

impl CompiledRule {
    /// Compile one rule, or `None` when it cannot be evaluated at all: a
    /// `when` or `assert` expression that fails to parse, or a rule naming
    /// both/neither of `assert`/`check`.
    ///
    /// A broken half drops the whole rule rather than half of it: an
    /// unparseable `when` left in place would silently widen the rule to every
    /// task, and an unparseable `assert` would fail every task the `when`
    /// admits. [`validate_rules`] rejects both at config load, so a `None` here
    /// only covers a hand-built config that never went through `load_config` —
    /// and a rule that cannot be evaluated constrains nothing, the same reading
    /// `required` and `[[tasks.body.headers.rules]] when` already take.
    fn compile(spec: &RuleSpec, index: usize, config: &Config) -> Option<Self> {
        let when = spec.when.as_deref().map(Filter::parse).transpose().ok()?;
        let kind = match (&spec.assertion, &spec.check) {
            (Some(expr), None) => RuleKind::Assert {
                filter: Filter::parse(expr).ok()?,
                advice: spec.advice(),
            },
            (None, Some(check)) => RuleKind::Check(CompiledCheck::new(
                check,
                config.rules_root.as_path(),
                config,
            )),
            (Some(_), Some(_)) | (None, None) => return None,
        };
        Some(Self {
            label: spec.label(index),
            when,
            kind,
            severity: spec.severity,
        })
    }

    /// Every breach `task` commits against this rule: zero or one for
    /// `assert`, zero or more for `check` (one per string its function
    /// returns).
    ///
    /// An `assert` that fails to evaluate reports as a breach naming the
    /// failure rather than passing quietly: the load-time dry run should have
    /// caught it, so reaching here means the rule is broken in a way only this
    /// task reveals, and a silently skipped assertion is the one outcome a
    /// project reading its own `mt check` output cannot detect. A `when` that
    /// fails to evaluate excludes the task instead — a gate that cannot say
    /// whether the rule applies has not said that it does.
    ///
    /// Either one reading `changed()` where git cannot answer is neither: it is
    /// `Err`, and [`check_all`] skips the rule (see [`Self::skipped`]).
    fn breaches(
        &self,
        task: &Task,
        config: &Config,
        readiness: &Readiness,
    ) -> Result<Vec<Violation>, FilterError> {
        let applies = match self
            .when
            .as_ref()
            .map(|when| when.matches_with(task, config, readiness))
        {
            None | Some(Ok(true)) => true,
            Some(Err(err @ FilterError::Git { .. })) => return Err(err),
            Some(Ok(false) | Err(_)) => false,
        };
        if !applies {
            return Ok(Vec::new());
        }
        Ok(match &self.kind {
            RuleKind::Assert { filter, advice } => {
                match filter.matches_with(task, config, readiness) {
                    Ok(true) => Vec::new(),
                    Ok(false) => vec![self.violation(task, advice.clone())],
                    Err(err @ FilterError::Git { .. }) => return Err(err),
                    Err(err) => vec![self.violation(task, err.to_string())],
                }
            }
            RuleKind::Check(check) => check
                .evaluate(task, config)
                .into_iter()
                .map(|message| self.violation(task, message))
                .collect(),
        })
    }

    /// The one report of a rule skipped because git could not answer its
    /// `changed()`: always a warning, whatever the rule's own severity — a
    /// rule that cannot see the change set has not found a breach.
    fn skipped(&self, task: &Task, err: &FilterError) -> Violation {
        Violation {
            severity: Some(Severity::Warn),
            ..self.violation(task, format!("skipped: {err}"))
        }
    }

    /// One breach as a [`Violation`]: the rule's label as the field, its
    /// severity carried through, and the task's marker line as the location so
    /// the report is openable where the `task.with(…)` call sits.
    fn violation(&self, task: &Task, message: String) -> Violation {
        Violation {
            task_id: task.id.clone(),
            field: self.label.clone(),
            message,
            location: task.marker_location(),
            severity: Some(self.severity),
        }
    }
}

/// Evaluate every `[[tasks.rules]]` entry over `tasks`, appending one violation
/// per breach — task by task, rules in config order. A rule whose `changed()`
/// git cannot answer reports once as skipped and is not asked again.
///
/// Readiness for the `ready` / `blocked` predicates is computed over the whole
/// set passed in, so a rule can assert something about what is startable as
/// readily as about a field. It is computed only when there is a rule to spend
/// it on: a project declaring none pays nothing — the same reading that keeps
/// a `check` rule from building so much as a [`CompiledCheck`] (never mind
/// evaluating Typst) when `config.tasks.rules` is empty.
pub(crate) fn check_all(tasks: &[Task], config: &Config, out: &mut Vec<Violation>) {
    let rules: Vec<CompiledRule> = config
        .tasks
        .rules
        .iter()
        .enumerate()
        .filter_map(|(index, spec)| CompiledRule::compile(spec, index, config))
        .collect();
    if rules.is_empty() {
        return;
    }
    let readiness = Readiness::compute(tasks, config);
    let mut skipped = vec![false; rules.len()];
    for task in tasks {
        for (rule, skipped) in rules.iter().zip(skipped.iter_mut()) {
            if *skipped {
                continue;
            }
            match rule.breaches(task, config, &readiness) {
                Ok(found) => out.extend(found),
                Err(err) => {
                    *skipped = true;
                    out.push(rule.skipped(task, &err));
                }
            }
        }
    }
}

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

#[cfg(test)]
#[path = "config_rules_tests.rs"]
mod tests;
