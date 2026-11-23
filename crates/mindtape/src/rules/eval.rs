//! The engine behind `[[tasks.rules]]`: compile each declared entry into a
//! `CompiledRule` and ask it about every task, collecting the breaches.
//!
//! Split out of `rules/mod.rs` to stay under the line budget. That file owns
//! the declared shape ([`RuleSpec`], [`CheckSpec`], [`RulesRoot`]) and the
//! load-time checks over it. This one owns what a valid declaration does at
//! validation time. Both shapes of rule meet here: an `assert` compiles to a
//! [`crate::filter::Filter`] and a `check` to a
//! [`crate::rules::typst::CompiledCheck`], and [`check_all`] builds neither
//! when the project has no rules.

use std::collections::BTreeSet;

use crate::config::Config;
use crate::filter::{Filter, FilterError};
use crate::graph::ready::Readiness;
use crate::model::Task;
use crate::rules::typst::CompiledCheck;
use crate::validate::Violation;
use crate::validate::severity::Severity;

use super::{MARKERS, RuleSpec};

/// The compiled form of a rule's own assertion, once the shape is clear.
enum RuleKind {
    /// `assert`: the parsed filter, plus the breach message to report on a
    /// `false` evaluation.
    Assert { filter: Filter, advice: String },
    /// `check`: the Typst function to call, reused across every task the rule
    /// covers.
    Check(CompiledCheck),
}

/// The names a rule reads are its `when` and `assert`, following a view either
/// names, or the `field` its `check` receives. A `check` handed the whole task
/// may be about any field, so it has no name set at all.
///
/// The reserved [`MARKERS`] input is not a task field, so a check over it
/// leaves the set to its `when` and `assert` names alone.
pub(super) fn rule_reads(
    when: Option<&str>,
    assertion: Option<&str>,
    check: Option<&crate::rules::CheckSpec>,
    config: &Config,
) -> Option<BTreeSet<String>> {
    let mut reads = BTreeSet::new();
    for expression in [when, assertion].into_iter().flatten() {
        reads.extend(crate::filter::names_in(expression, config));
    }
    if let Some(check) = check {
        // A `check` handed the whole task may be about any field.
        let field = check.field.as_deref()?;
        // The reserved input is not a task field, so it stays out of the set.
        reads.extend((field != MARKERS).then(|| field.to_owned()));
    }
    Some(reads)
}

/// One `[[tasks.rules]]` entry with its shape compiled, ready to ask task by
/// task.
struct CompiledRule {
    label: String,
    when: Option<Filter>,
    kind: RuleKind,
    severity: Severity,
    /// What the rule reads, for [`Violation::reads`].
    reads: Option<BTreeSet<String>>,
}

impl CompiledRule {
    /// Compile one rule, or `None` when the rule cannot run at all. That covers
    /// a `when` or `assert` expression that fails to parse, and a rule that
    /// declares both or neither of `assert`/`check`.
    ///
    /// A broken half drops the whole rule rather than half of it. An
    /// unparseable `when` left in place would silently widen the rule to every
    /// task, and an unparseable `assert` would fail every task the `when`
    /// admits. [`super::validate_rules`] rejects both at config load, so a
    /// `None` here only covers a hand-built config that never went through
    /// `load_config`. A rule that cannot run constrains nothing, the same
    /// reading `required` and `[[tasks.body.headers.rules]] when` already take.
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
        let reads = rule_reads(
            spec.when.as_deref(),
            spec.assertion.as_deref(),
            spec.check.as_ref(),
            config,
        );
        Some(Self {
            label: spec.label(index),
            when,
            kind,
            severity: spec.severity,
            reads,
        })
    }

    /// Every breach `task` commits against this rule: zero or one for
    /// `assert`, zero or more for `check` (one per string its function
    /// returns).
    ///
    /// An `assert` that fails to evaluate reports as a breach carrying the
    /// failure rather than passing quietly. The load-time dry run should have
    /// caught it, so reaching here means the rule broke in a way only this task
    /// reveals. A silently skipped assertion is the one outcome a project
    /// reading its own `mt check` output cannot detect. A `when` that fails to
    /// evaluate excludes the task instead: a gate that cannot say whether the
    /// rule applies has not said that it does.
    ///
    /// Either one reading `changed()` where git cannot answer is neither: it is
    /// `Err`, and [`check_all`] skips the rule (see [`Self::skipped`]).
    ///
    /// An `assert` that reads false carries the `in_dict()` miss behind it,
    /// if any ([`Filter::dict_miss`]), so the report can point at the
    /// dictionary.
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
                    Ok(false) => vec![Violation {
                        detail: None,
                        dict_miss: filter.dict_miss(task, config),
                        ..self.violation(task, advice.clone())
                    }],
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
    /// `changed()`. It is always a warning, whatever the rule's own severity. A
    /// rule that cannot see the change set has not found a breach.
    ///
    /// Placed at the rule's own entry, in whichever config layer or pack
    /// declared it: the rule is what is at fault, not the task it happened to
    /// meet first. A config built without a load has no entry to name, and the
    /// report stays on the task's marker.
    fn skipped(&self, task: &Task, err: &FilterError, config: &Config) -> Violation {
        let found = self.violation(task, format!("skipped: {err}"));
        Violation {
            severity: Severity::Warn,
            kind: crate::validate::violation::ViolationKind::SkippedRule,
            location: config
                .rule_sites
                .get(&self.label)
                .cloned()
                .unwrap_or(found.location.clone()),
            ..found
        }
    }

    /// One breach as a [`Violation`]: the rule's label as the field, its
    /// severity carried through, and the task's marker line as the location.
    /// The report then opens where the `task.with(…)` call sits.
    fn violation(&self, task: &Task, message: String) -> Violation {
        Violation {
            task_id: task.id.clone(),
            field: self.label.clone(),
            message,
            location: task.marker_location(),
            severity: self.severity,
            kind: crate::validate::violation::ViolationKind::Other,
            reads: self.reads.clone(),
            unknown: None,
            detail: None,
            dict_miss: None,
        }
    }
}

/// Evaluate every `[[tasks.rules]]` entry over `tasks`, appending one violation
/// per breach, task by task, rules in config order. A rule whose `changed()`
/// git cannot answer reports once as skipped and is not asked again.
///
/// `check_all` computes readiness for the `ready` / `blocked` predicates over
/// the whole set passed in. A rule can then assert something about what is
/// startable as readily as about a field. It computes readiness only when there
/// is a rule to spend it on. A project with no rules skips that work. The same
/// reading keeps a `check` rule from building so much as a [`CompiledCheck`]
/// (never mind evaluating Typst) when `config.tasks.rules` is empty.
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
                    out.push(rule.skipped(task, &err, config));
                }
            }
        }
    }
}
