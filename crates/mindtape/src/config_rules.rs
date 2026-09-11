//! `[[tasks.rules]]` — project-declared assertions over every task.
//!
//! Every other constraint `MindTape` enforces is one it knows by name: a status
//! legend, a numeric scale, a tag allow-list, a heading path. A rule is the
//! general form — an assertion written in the filter language that must hold
//! for every task the rule selects. It constrains no field in particular,
//! because the expression already names the field it is about:
//!
//! ```toml
//! [[tasks.rules]]
//! name = "imperative-title"
//! assert = 'in_dict("verbs", word(title, 1))'
//! when = '!terminal'
//! message = "start the title with an imperative verb"
//! severity = "error"
//! ```
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
//! Both expressions are dry-run once at config load ([`validate_rules`]), the
//! way `required` and `[[tasks.limits]]` are, so a malformed rule is one config
//! error rather than one violation per artifact.

use serde::Deserialize;

use crate::config::Config;
use crate::filter::Filter;
use crate::model::Task;
use crate::ready::Readiness;
use crate::severity::Severity;
use crate::validate::Violation;

/// `[[tasks.rules]]` — one assertion every selected task must satisfy.
///
/// `assert` is the only required key: a rule with nothing to assert is not a
/// rule, and an absent one cannot be defaulted to anything that constrains.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RuleSpec {
    /// What the rule is called, used to report it (`rules.<name>`). Optional:
    /// an unnamed rule reports by its position instead (`rules[0]`).
    #[serde(default)]
    pub name: Option<String>,
    /// Filter expression that must evaluate true for every task the rule
    /// applies to.
    #[serde(rename = "assert")]
    pub assertion: String,
    /// Filter expression gating which tasks the rule applies to. Absent means
    /// every task.
    #[serde(default)]
    pub when: Option<String>,
    /// What a breach tells the author to do. Defaults to naming the `assert`
    /// expression itself.
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

    /// What a breach of this rule tells the author.
    fn advice(&self) -> String {
        self.message.clone().unwrap_or_else(|| {
            let expr = &self.assertion;
            format!("must satisfy `{expr}`")
        })
    }
}

/// One `[[tasks.rules]]` entry with both its expressions parsed, ready to ask
/// task by task.
struct CompiledRule {
    label: String,
    advice: String,
    when: Option<Filter>,
    assertion: Filter,
    severity: Severity,
}

impl CompiledRule {
    /// Compile one rule, or `None` when either expression fails to parse.
    ///
    /// A broken half drops the whole rule rather than half of it: an
    /// unparseable `when` left in place would silently widen the rule to every
    /// task, and an unparseable `assert` would fail every task the `when`
    /// admits. [`validate_rules`] rejects both at config load, so a `None` here
    /// only covers a hand-built config that never went through `load_config` —
    /// and a rule that cannot be evaluated constrains nothing, the same reading
    /// `required` and `[[tasks.body.headers.rules]] when` already take.
    fn compile(spec: &RuleSpec, index: usize) -> Option<Self> {
        Some(Self {
            label: spec.label(index),
            advice: spec.advice(),
            when: spec.when.as_deref().map(Filter::parse).transpose().ok()?,
            assertion: Filter::parse(&spec.assertion).ok()?,
            severity: spec.severity,
        })
    }

    /// The breach `task` commits against this rule, if any.
    ///
    /// An `assert` that fails to evaluate reports as a breach naming the
    /// failure rather than passing quietly: the load-time dry run should have
    /// caught it, so reaching here means the rule is broken in a way only this
    /// task reveals, and a silently skipped assertion is the one outcome a
    /// project reading its own `mt check` output cannot detect. A `when` that
    /// fails to evaluate excludes the task instead — a gate that cannot say
    /// whether the rule applies has not said that it does.
    fn breach(&self, task: &Task, config: &Config, readiness: &Readiness) -> Option<Violation> {
        let applies = self
            .when
            .as_ref()
            .is_none_or(|when| when.matches_with(task, config, readiness).unwrap_or(false));
        if !applies {
            return None;
        }
        match self.assertion.matches_with(task, config, readiness) {
            Ok(true) => None,
            Ok(false) => Some(self.violation(task, self.advice.clone())),
            Err(err) => Some(self.violation(task, err.to_string())),
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
/// per breach — task by task, rules in config order.
///
/// Readiness for the `ready` / `blocked` predicates is computed over the whole
/// set passed in, so a rule can assert something about what is startable as
/// readily as about a field. It is computed only when there is a rule to spend
/// it on: a project declaring none pays nothing.
pub(crate) fn check_all(tasks: &[Task], config: &Config, out: &mut Vec<Violation>) {
    let rules: Vec<CompiledRule> = config
        .tasks
        .rules
        .iter()
        .enumerate()
        .filter_map(|(index, spec)| CompiledRule::compile(spec, index))
        .collect();
    if rules.is_empty() {
        return;
    }
    let readiness = Readiness::compute(tasks, config);
    for task in tasks {
        out.extend(
            rules
                .iter()
                .filter_map(|rule| rule.breach(task, config, &readiness)),
        );
    }
}

/// Check every `[[tasks.rules]]` expression at config load, returning one
/// message per unusable one, prefixed with its dotted TOML path.
///
/// Both `assert` and `when` are parsed and dry-run against a probe task, so an
/// unknown field, view or dictionary is caught here — where it is one config
/// error — instead of once per artifact at validation time.
pub(crate) fn validate_rules(config: &Config) -> Vec<String> {
    let probe = Task::default();
    let mut problems = Vec::new();
    for (index, spec) in config.tasks.rules.iter().enumerate() {
        let mut report = |key: &str, source: &str| {
            if let Some(problem) = crate::filter::expr_problem(source, &probe, config) {
                problems.push(format!("tasks.rules[{index}].{key}: {problem}"));
            }
        };
        report("assert", &spec.assertion);
        if let Some(when) = &spec.when {
            report("when", when);
        }
    }
    problems
}

#[cfg(test)]
#[path = "config_rules_tests.rs"]
mod tests;
