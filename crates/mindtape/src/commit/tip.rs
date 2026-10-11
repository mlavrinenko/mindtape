//! The `scope = "tip"` commit rules of a range: each runs once per task any
//! commit of the range names, over that task as the range's newest commit
//! holds it.
//!
//! A tip rule is a `[[tasks.rules]]` expression, not a commit expression: at
//! the tip there is one task state and no single commit, so `status`,
//! `ready` and `count(...)` read as they do in `mt ls`. A task the tip does
//! not hold (the range deleted or retired it) has no state to be wrong in,
//! so it is skipped. A rule that [reads the working tree](crate::CommitRuleSpec::reads_worktree)
//! is skipped too, since a past tip has none.

use std::collections::BTreeSet;

use crate::commit::tasks::ProjectState;
use crate::config::Config;
use crate::config::commits::{CommitBreach, CommitScope};
use crate::filter::Filter;

/// Evaluate every `scope = "tip"` rule once per distinct stem in `stems`,
/// reading each task from `tip`, in rule order and then stem order.
///
/// A `when` that fails to evaluate excludes the task; an `assert` that
/// fails to evaluate is a breach naming the failure, as in
/// [`check_commit`](crate::check_commit).
#[must_use]
pub fn check_tip<'a>(
    stems: impl IntoIterator<Item = &'a str>,
    tip: &ProjectState,
    config: &Config,
) -> Vec<CommitBreach> {
    let stems: BTreeSet<&str> = stems.into_iter().collect();
    let mut breaches = Vec::new();
    for (index, spec) in config.commits.rules.iter().enumerate() {
        if spec.scope != CommitScope::Tip || spec.reads_worktree() {
            continue;
        }
        let when = spec.when.as_deref().map(parse);
        let assertion = parse(&spec.assertion);
        for stem in &stems {
            let Some(task) = tip.task(stem) else { continue };
            let read = |filter: &Result<Filter, String>| match filter {
                Ok(filter) => filter
                    .matches_with(task, config, &tip.readiness)
                    .map_err(|err| err.to_string()),
                Err(err) => Err(err.clone()),
            };
            if when.as_ref().is_none_or(|when| read(when).unwrap_or(false)) {
                breaches.extend(spec.breach(index, Some(stem), read(&assertion)));
            }
        }
    }
    breaches
}

fn parse(source: &str) -> Result<Filter, String> {
    Filter::parse(source).map_err(|err| err.to_string())
}

#[cfg(test)]
#[path = "tip_tests.rs"]
mod tests;
