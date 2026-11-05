//! An estimate unit resolved from a manual count plus file queries.
//!
//! ```typ
//! estimate: (src-files: count(2, grep(`parse_task`, "src/**/*.rs"))),
//! ```
//!
//! The prelude's `count(n, ..queries)` builds a `(count, queries)` dict, and
//! `grep(regex, glob)`, `glob(pattern)` and
//! `ast_grep(language, pattern, glob)` build one query each. A unit takes
//! `count(...)` only when `[tasks.estimate.queries]` names the path set its
//! matches must fall in:
//!
//! ```toml
//! [tasks.estimate.queries]
//! src-files = "@source"
//! ```
//!
//! [`annotate`] resolves each such unit to `n` plus the number of files any of
//! its queries match, a file once however many queries match it, and writes
//! that number into [`Task::estimate`]. So a rule reads `estimate.src-files`
//! as it reads a plain number. A query that can't run leaves the unit out of
//! [`Task::estimate`] and records why in [`CountEstimate::problem`], which
//! [`crate::validate`] reports.
//!
//! The files come from [`crate::query`], the shared engine this module is the
//! first consumer of. It lists the files git sees, narrows each query to the
//! unit's path set and runs each distinct query once per process.

use std::collections::BTreeMap;
use std::path::Path;

use typst_harvest::HVal;

use crate::config::Config;
use crate::model::Task;
use crate::model::marker::{ShapeError, shape_error, shape_message};
use crate::problem::Problem;
use crate::query::{self, Query};

/// An estimate unit written as `count(n, ..queries)`.
#[derive(Debug, Clone, PartialEq)]
pub struct CountEstimate {
    /// The manual part, `n`.
    pub manual: f64,
    /// The queries whose matched files add to it, in authored order.
    pub queries: Vec<Query>,
    /// How many files each query matched, in the order of
    /// [`queries`](Self::queries), set by [`annotate`]. Their sum can pass
    /// the resolved count, since a file several queries match counts once.
    pub matches: Vec<usize>,
    /// Why the unit didn't resolve, set by [`annotate`]; `None` once it did.
    pub problem: Option<String>,
}

/// Project the `estimate` field: a plain number per unit, or a
/// `count(...)` dict. A value that is neither is a [`ShapeError`] at
/// `estimate.<unit>`.
pub(crate) fn parse_estimate(
    value: Option<&HVal>,
    errors: &mut Vec<ShapeError>,
) -> (BTreeMap<String, f64>, BTreeMap<String, CountEstimate>) {
    let mut plain = BTreeMap::new();
    let mut counts = BTreeMap::new();
    let Some(value) = value else {
        return (plain, counts);
    };
    let Some(dict) = value.as_dict() else {
        errors.push(shape_error("estimate", "a dict", value));
        return (plain, counts);
    };
    for (unit, val) in dict.iter().filter(|(_, val)| !val.is_null()) {
        let at = format!("estimate.{unit}");
        if let Some(num) = val.as_f64() {
            plain.insert(unit.clone(), num);
        } else if val.get("count").is_some() {
            if let Some(count) = parse_count(val, &at, errors) {
                counts.insert(unit.clone(), count);
            }
        } else {
            errors.push(shape_error(&at, "a number or count(...)", val));
        }
    }
    (plain, counts)
}

/// Read one `(count, queries)` dict, reporting a bad part at `at`.
fn parse_count(value: &HVal, at: &str, errors: &mut Vec<ShapeError>) -> Option<CountEstimate> {
    let Some(manual) = value.get("count").and_then(HVal::as_f64) else {
        errors.push(shape_message(at, "count(...) takes a number first"));
        return None;
    };
    let items = value
        .get("queries")
        .and_then(HVal::as_array)
        .unwrap_or_default();
    let mut queries = Vec::new();
    for (index, item) in items.iter().enumerate() {
        if item
            .get("glob")
            .filter(|glob| !glob.is_null())
            .is_some_and(|glob| glob.as_str().is_none())
        {
            errors.push(shape_message(
                &format!("{at}.queries[{index}]"),
                "glob must be a string",
            ));
            continue;
        }
        match query::parse(item) {
            Some(query) => queries.push(query),
            None => errors.push(shape_message(
                &format!("{at}.queries[{index}]"),
                "expected grep(...), glob(...) or ast_grep(...)",
            )),
        }
    }
    Some(CountEstimate {
        manual,
        queries,
        matches: Vec::new(),
        problem: None,
    })
}

/// Resolve every `count(...)` unit of every task in `tasks` against the files
/// git lists under `root`: see the module docs.
pub fn annotate(tasks: &mut [Task], config: &Config, root: &Path) {
    for task in tasks {
        for (unit, count) in &mut task.estimate_counts {
            match resolve(unit, count, config, root) {
                Ok((union, matches)) => {
                    count.matches = matches;
                    count.problem = None;
                    task.estimate
                        .insert(unit.clone(), count.manual + union as f64);
                }
                Err(problem) => {
                    count.matches.clear();
                    count.problem = Some(problem);
                    task.estimate.remove(unit);
                }
            }
        }
    }
}

/// How many files the queries of `count` match inside `unit`'s path set, a
/// file once, and how many each query matched.
fn resolve(
    unit: &str,
    count: &CountEstimate,
    config: &Config,
    root: &Path,
) -> Result<(usize, Vec<usize>), String> {
    let set = config.tasks.estimate.queries.get(unit).ok_or_else(|| {
        format!(
            "`{unit}` takes a plain number; name its path set under [tasks.estimate.queries] to count files"
        )
    })?;
    let dicts = config.compiled_dicts();
    let admitted = query::path_filter(set, dicts).map_err(problem_text)?;
    let mut union = std::collections::BTreeSet::new();
    let mut each = Vec::new();
    for query in &count.queries {
        let matched = query::matches(root, set, query, &admitted, dicts).map_err(problem_text)?;
        each.push(matched.len());
        union.extend(matched.iter().cloned());
    }
    Ok((union.len(), each))
}

/// A query problem as the one-line text an estimate keeps, with the nearest
/// `@name` path set appended where a `Problem` would carry it as a hint.
fn problem_text(problem: Problem) -> String {
    match problem
        .unknown
        .as_ref()
        .and_then(|unknown| unknown.nearest.as_deref())
    {
        Some(nearest) => format!("{problem}; did you mean `{nearest}`?"),
        None => problem.text,
    }
}

/// Config-load problems in `[tasks.estimate.queries]`: a unit outside a
/// declared `units` list, or a path set that doesn't resolve.
pub(crate) fn validate_config(config: &Config) -> Vec<Problem> {
    let estimate = &config.tasks.estimate;
    let mut problems = Vec::new();
    for (unit, set) in &estimate.queries {
        let at = format!("tasks.estimate.queries.{unit}");
        if !estimate.units.is_empty() && !estimate.units.contains(unit) {
            problems.push(Problem::new(format!(
                "{at}: `{unit}` is not an allowed estimate unit"
            )));
        }
        if let Err(problem) = query::path_filter(set, config.compiled_dicts()) {
            problems.push(problem.under(&at));
        }
    }
    problems
}

#[cfg(test)]
#[path = "estimate_queries_tests.rs"]
mod tests;
