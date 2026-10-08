//! The load-time checks on a saved filter expression: what makes it unusable
//! ([`expr_problem`], [`expr_problem_in_set`]) and which `x-` names it reads
//! that no field declares ([`undeclared_extras`]). Split out of `filter/mod.rs`
//! for the file-size gate.

use super::{Filter, Readiness};
use crate::config::Config;
use crate::model::Task;
use crate::model::fields;
use crate::problem::Problem;
use crate::suggest::Unknown;

/// What makes one saved filter expression unusable, if anything: a syntax
/// error, or a structural problem a dry-run against `probe` surfaces — an
/// unknown field, an unknown view, an unknown dictionary, an invalid regex.
///
/// The load-time half of every config section that stores an expression
/// (`required`, `[[tasks.limits]]`, `[[tasks.rules]]`). Each of those calls it
/// once per expression from [`crate::config::Config::validate`] and prefixes
/// the answer with its own dotted TOML path, so a malformed expression is one
/// config error at load rather than a surprise per artifact — or, for a rule,
/// one violation per artifact.
///
/// The dry run never reads git: a `changed()` call has its states and globs
/// checked and counts zero, so loading a config outside a repository is not
/// an error and loading one inside it costs no `git status`.
#[must_use]
pub(crate) fn expr_problem(source: &str, probe: &Task, config: &Config) -> Option<Problem> {
    match Filter::parse(source) {
        Err(err) => Some(problem_of(&err)),
        Ok(filter) => filter
            .eval(probe, config, &Readiness::empty(), true)
            .err()
            .map(|err| problem_of(&err)),
    }
}

/// `err` as a problem, carrying the dictionary names an unknown `in_dict`
/// could have meant.
pub(crate) fn problem_of(err: &super::FilterError) -> Problem {
    let problem = Problem::new(err.to_string());
    match err {
        super::FilterError::UnknownDict { unknown, .. }
        | super::FilterError::UnknownPathSet { unknown, .. } => problem.with(unknown.clone()),
        _ => problem,
    }
}

/// The `x-` names `source` reads that `[tasks.fields]` does not declare, each
/// as a warning with the nearest declared name. Empty while the config declares
/// no `x-` field at all, and for an expression that does not parse: a misspelt
/// name is a likely typo only beside declared ones, and reading an undeclared
/// name stays legal.
#[must_use]
pub(crate) fn undeclared_extras(source: &str, config: &Config) -> Vec<Problem> {
    let declared: Vec<String> = config
        .tasks
        .fields
        .keys()
        .filter(|name| fields::is_extra(name))
        .cloned()
        .collect();
    let Ok(filter) = Filter::parse(source) else {
        return Vec::new();
    };
    if declared.is_empty() {
        return Vec::new();
    }
    filter
        .names()
        .into_iter()
        .filter(|name| fields::is_extra(name) && !declared.contains(name))
        .map(|name| {
            Problem::new(format!("`{name}` is not declared in [tasks.fields]"))
                .with(Unknown::among(&name, declared.iter().map(String::as_str)))
        })
        .collect()
}

/// [`expr_problem`] for an expression evaluated over the loaded set — a
/// `[[tasks.rules]]` entry, an `[[on.flip]]` hook — so `count(...)` passes the
/// dry run: the probe is its own one-task set. The run is dry, so loading
/// never reads git for `changed()` or `touches()`.
#[must_use]
pub(crate) fn expr_problem_in_set(source: &str, probe: &Task, config: &Config) -> Option<Problem> {
    let set = Readiness::compute(std::slice::from_ref(probe), config);
    match Filter::parse(source) {
        Err(err) => Some(problem_of(&err)),
        Ok(filter) => filter
            .eval(probe, config, &set, true)
            .err()
            .map(|err| problem_of(&err)),
    }
}
