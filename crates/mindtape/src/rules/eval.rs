//! The engine behind `[[tasks.rules]]`: compile each declared entry into a
//! [`CompiledRule`] and ask it about every task, collecting the breaches.
//!
//! Split out of `rules/mod.rs` to stay under the line budget: that file
//! owns the declared shape — [`RuleSpec`], [`CheckSpec`], [`RulesRoot`] — and
//! the load-time checks over it; this one owns what a valid declaration does
//! at validation time. Both shapes of rule meet here: an `assert` compiles to
//! a [`Filter`] and a `check` to a [`CompiledCheck`], and [`check_all`] builds
//! neither when the project declares no rules.

use std::collections::BTreeSet;

use crate::config::Config;
use crate::filter::{Filter, FilterError};
use crate::model::Task;
use crate::ready::Readiness;
use crate::rules::typst::CompiledCheck;
use crate::severity::Severity;
use crate::validate::Violation;

use super::RuleSpec;

/// The compiled form of a rule's own assertion, once its shape is known.
enum RuleKind {
    /// `assert`: the parsed filter, plus the breach message to report on a
    /// `false` evaluation.
    Assert { filter: Filter, advice: String },
    /// `check`: the Typst function to call, reused across every task this
    /// rule is asked about.
    Check(CompiledCheck),
}

/// The names a rule reads: its `when` and `assert`, or the `field` its `check`
/// receives. A `check` handed the whole task, or an expression reaching
/// through a view, may be about any field.
fn rule_reads(spec: &RuleSpec, when: Option<&Filter>, kind: &RuleKind) -> Option<BTreeSet<String>> {
    let mut reads = crate::violation::reads_of(when)?;
    match kind {
        RuleKind::Assert { filter, .. } => reads.extend(crate::violation::reads_of([filter])?),
        RuleKind::Check(_) => {
            reads.insert(spec.check.as_ref()?.field.clone()?);
        }
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
        let reads = rule_reads(spec, when.as_ref(), &kind);
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
    ///
    /// Placed at the rule's own entry, in whichever config layer or pack
    /// declared it: the rule is what is at fault, not the task it happened to
    /// meet first. A config built without a load has no entry to name, and the
    /// report stays on the task's marker.
    fn skipped(&self, task: &Task, err: &FilterError, config: &Config) -> Violation {
        let found = self.violation(task, format!("skipped: {err}"));
        Violation {
            severity: Severity::Warn,
            kind: crate::violation::ViolationKind::SkippedRule,
            location: config
                .rule_sites
                .get(&self.label)
                .cloned()
                .unwrap_or(found.location.clone()),
            ..found
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
            severity: self.severity,
            kind: crate::violation::ViolationKind::Other,
            reads: self.reads.clone(),
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
                    out.push(rule.skipped(task, &err, config));
                }
            }
        }
    }
}
