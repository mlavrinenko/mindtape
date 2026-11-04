//! Run the project's `[[tasks.collectors]]` and write each result into
//! [`Task::derived`].
//!
//! A collector runs after a task is loaded and after the whole-set facts are
//! known, because its `when` may read `ready`, `blocked` or `count(...)`. A
//! regex collector reads a body section and returns one string per match: the
//! pattern's first capture group when it has one, else the whole match, or the
//! capture group `group` names. A missing section is an empty list. A task the
//! `when` excludes carries no field at all.
//!
//! The section slice is the one [`crate::body::content_gate`] uses: everything
//! from the end of the heading down to the next heading of the same or higher
//! level. Collectors run in declaration order, so a later collector's `when`
//! may read an earlier one's field.
//!
//! A per-task failure — the Typst kind's evaluation error or a wrong-typed
//! result — is recorded on the task and reported as a `Collector` violation by
//! [`crate::validate`]. The regex kind fails only at load, where its pattern,
//! group and `returns` are checked ([`crate::config::collect`]).

use regex::Regex;
use typst_cst::{parse_headings, tree};
use typst_harvest::HVal;

use crate::config::collect::{CollectorSpec, CollectorType, RegexQuery};
use crate::config::{CollectorQuery, Config};
use crate::filter::Filter;
use crate::graph::ready::Readiness;
use crate::model::Task;

/// A collector compiled once for a run: the spec, its parsed `when`, and the
/// compiled regex of a regex query.
struct Compiled<'a> {
    spec: &'a CollectorSpec,
    when: Option<Filter>,
    regex: Option<Regex>,
}

impl<'a> Compiled<'a> {
    fn new(spec: &'a CollectorSpec) -> Self {
        let when = spec
            .when
            .as_deref()
            .and_then(|source| Filter::parse(source).ok());
        let regex = match &spec.query {
            CollectorQuery::Regex(query) => Regex::new(&query.pattern).ok(),
        };
        Self { spec, when, regex }
    }

    fn applies(&self, task: &Task, config: &Config, readiness: &Readiness) -> bool {
        self.when.as_ref().is_none_or(|filter| {
            filter
                .matches_with(task, config, readiness)
                .unwrap_or(false)
        })
    }

    fn value(&self, task: &Task) -> Option<HVal> {
        match &self.spec.query {
            CollectorQuery::Regex(query) => {
                let regex = self.regex.as_ref()?;
                Some(HVal::Array(regex_values(task, query, regex)))
            }
        }
    }
}

/// Run every collector over `tasks`, in declaration order, and write each
/// result into the task's [`derived`](Task::derived) map.
///
/// `readiness` is computed over the same slice, so a `when` that reads the
/// set sees every task of the load. A task the collector's `when` excludes
/// keeps no field for that collector.
pub fn annotate(tasks: &mut [Task], config: &Config) {
    if config.tasks.collectors.is_empty() {
        return;
    }
    let compiled: Vec<Compiled<'_>> = config.tasks.collectors.iter().map(Compiled::new).collect();
    let readiness = Readiness::compute(tasks, config);
    for task in tasks.iter_mut() {
        for collector in &compiled {
            if !collector.applies(task, config, &readiness) {
                continue;
            }
            if let Some(value) = collector.value(task) {
                task.derived.insert(collector.spec.name.clone(), value);
            }
        }
    }
}

/// The strings a regex query produces: every match in every selected section,
/// in section order then match order.
fn regex_values(task: &Task, query: &RegexQuery, regex: &Regex) -> Vec<HVal> {
    let group = query
        .group
        .unwrap_or_else(|| usize::from(regex.captures_len() > 1));
    let roots = tree(parse_headings(&task.body));
    let mut values = Vec::new();
    for node in artifact_gate::select_by(
        &roots,
        &crate::body::gate::compile_path(&query.section.path()).unwrap_or_default(),
        |node| node.heading.text.as_str(),
        |node| node.children.as_slice(),
    ) {
        let content = task
            .body
            .get(node.heading.section.clone())
            .unwrap_or_default();
        for captures in regex.captures_iter(content) {
            if let Some(found) = captures.get(group) {
                values.push(HVal::Str(found.as_str().to_owned()));
            }
        }
    }
    values
}

/// Whether `value` matches the collector's declared `returns` type.
#[must_use]
pub fn matches_type(returns: &CollectorType, value: &HVal) -> bool {
    returns.accepts(value)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
