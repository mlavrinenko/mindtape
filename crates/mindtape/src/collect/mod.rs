//! Run the project's `[[tasks.collectors]]` and write each result into
//! [`Task::derived`].
//!
//! A collector runs after a task is loaded and after the whole-set facts are
//! known, because its `when` may read `ready`, `blocked` or `count(...)`. A
//! regex collector reads a body section and returns one string per match: the
//! pattern's first capture group when it has one, else the whole match, or the
//! capture group `group` names. A missing section is an empty list. An
//! ast-grep collector reuses [`crate::query`] and returns one path per file it
//! matches. A task the `when` excludes carries no field at all.
//!
//! The section slice is the one [`crate::body::content_gate`] uses: everything
//! from the end of the heading down to the next heading of the same or higher
//! level. Collectors run in declaration order, so a later collector's `when`
//! may read an earlier one's field.
//!
//! A per-task failure — the Typst kind's evaluation error, a wrong-typed
//! result or a file query that cannot run — is recorded on the task and
//! reported as a `Collector` violation by [`crate::validate`]. The regex and
//! ast-grep kinds fail at load, where their pattern, group, language, glob and
//! `returns` are checked ([`crate::config::collect`]).

use regex::Regex;
use typst_cst::{parse_headings, tree};
use typst_harvest::HVal;

use crate::config::collect::{CollectorSpec, RegexQuery};
use crate::config::{CollectorQuery, Config};
use crate::filter::Filter;
use crate::graph::ready::Readiness;
use crate::model::Task;

mod astgrep;
mod typst;
use typst::CompiledTypst;

/// A collector compiled once for a run: the spec, its parsed `when`, and the
/// compiled query.
struct Compiled<'a> {
    spec: &'a CollectorSpec,
    when: Option<Filter>,
    query: Option<CompiledQuery>,
}

/// A compiled regex or Typst query, or the marker for an ast-grep query the
/// shared engine caches itself.
enum CompiledQuery {
    Regex(Regex),
    Typst(Box<CompiledTypst>),
    AstGrep,
}

impl<'a> Compiled<'a> {
    fn new(spec: &'a CollectorSpec, root: &std::path::Path, config: &Config) -> Self {
        let when = spec
            .when
            .as_deref()
            .and_then(|source| Filter::parse(source).ok());
        let query = match &spec.query {
            CollectorQuery::Regex(query) => {
                Regex::new(&query.pattern).ok().map(CompiledQuery::Regex)
            }
            CollectorQuery::Typst(query) => Some(CompiledQuery::Typst(Box::new(
                CompiledTypst::new(spec, query, root, config),
            ))),
            CollectorQuery::AstGrep(_) => Some(CompiledQuery::AstGrep),
        };
        Self { spec, when, query }
    }

    fn applies(&self, task: &Task, config: &Config, readiness: &Readiness) -> bool {
        self.when.as_ref().is_none_or(|filter| {
            filter
                .matches_with(task, config, readiness)
                .unwrap_or(false)
        })
    }

    fn value(&self, task: &mut Task, config: &Config) -> Option<HVal> {
        match (&self.spec.query, &self.query) {
            (CollectorQuery::Regex(query), Some(CompiledQuery::Regex(regex))) => {
                Some(HVal::Array(regex_values(task, query, regex)))
            }
            (CollectorQuery::Typst(_), Some(CompiledQuery::Typst(compiled))) => {
                match compiled.evaluate(task, config) {
                    Ok(value) => Some(value),
                    Err(problem) => {
                        task.collector_problems.push(problem);
                        None
                    }
                }
            }
            (CollectorQuery::AstGrep(query), Some(CompiledQuery::AstGrep)) => {
                match astgrep::values(&self.spec.name, query, config) {
                    Ok(values) => Some(HVal::Array(values)),
                    Err(problem) => {
                        task.collector_problems.push(problem);
                        None
                    }
                }
            }
            _ => None,
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
    for task in tasks.iter_mut() {
        task.derived.clear();
        task.collector_problems.clear();
    }
    if config.tasks.collectors.is_empty() {
        return;
    }
    let root = config.rules_root.as_path();
    let compiled: Vec<Compiled<'_>> = config
        .tasks
        .collectors
        .iter()
        .map(|spec| Compiled::new(spec, root, config))
        .collect();
    let readiness = Readiness::compute(tasks, config);
    for task in tasks.iter_mut() {
        for collector in &compiled {
            if !collector.applies(task, config, &readiness) {
                continue;
            }
            if let Some(value) = collector.value(task, config) {
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "typst_tests.rs"]
mod typst_tests;
