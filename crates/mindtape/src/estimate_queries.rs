//! An estimate unit resolved from a manual count plus file queries.
//!
//! ```typ
//! estimate: (src-files: count(2, grep(`parse_task`, within: "src/**/*.rs"))),
//! ```
//!
//! The prelude's `count(n, ..queries)` builds a `(count, queries)` dict, and
//! `grep(regex, within: glob)` and `glob(pattern)` build one query each. A
//! unit takes `count(...)` only when `[tasks.estimate.queries]` names the path
//! set its matches must fall in:
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
//! A query sees the files git lists in the working tree: tracked ones, and
//! untracked ones `.gitignore` doesn't hide. Matching is native, `regex` over
//! file bytes and `globset` over paths, and never runs a shell. Paths are
//! relative to the repository root, as `changed()` reads them. Each project
//! root lists its files once per process, and each distinct query runs once.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use typst_harvest::HVal;

use crate::config::Config;
use crate::config::dicts::Dicts;
use crate::git::state::{PathFilter, PathSetError, git};
use crate::model::Task;
use crate::model::marker::{ShapeError, shape_error, shape_message};

/// One file query inside `count(...)`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Query {
    /// `grep(regex, within: glob)`: the files whose bytes match `regex`,
    /// narrowed to `within` when it is set.
    Grep {
        /// The pattern, in `regex` crate syntax.
        regex: String,
        /// A glob or `@name` path set the file must also fall in.
        within: Option<String>,
    },
    /// `glob(pattern)`: the files whose path matches `pattern`.
    Glob {
        /// A glob or `@name` path set.
        pattern: String,
    },
}

impl Query {
    /// The query as `mt` names it in a problem, such as ``grep `parse(` ``.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Grep { regex, .. } => format!("grep `{regex}`"),
            Self::Glob { pattern } => format!("glob `{pattern}`"),
        }
    }
}

/// An estimate unit written as `count(n, ..queries)`.
#[derive(Debug, Clone, PartialEq)]
pub struct CountEstimate {
    /// The manual part, `n`.
    pub manual: f64,
    /// The queries whose matched files add to it, in authored order.
    pub queries: Vec<Query>,
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
        match parse_query(item) {
            Some(query) => queries.push(query),
            None => errors.push(shape_message(
                &format!("{at}.queries[{index}]"),
                "expected grep(...) or glob(...)",
            )),
        }
    }
    Some(CountEstimate {
        manual,
        queries,
        problem: None,
    })
}

/// Read one query dict, or `None` when it is no query.
fn parse_query(value: &HVal) -> Option<Query> {
    let text = |key: &str| value.get(key).and_then(HVal::as_str).map(str::to_owned);
    match value.get("query").and_then(HVal::as_str)? {
        "grep" => Some(Query::Grep {
            regex: text("regex")?,
            within: text("within"),
        }),
        "glob" => Some(Query::Glob {
            pattern: text("pattern")?,
        }),
        _ => None,
    }
}

/// Resolve every `count(...)` unit of every task in `tasks` against the files
/// git lists under `root`: see the module docs.
pub fn annotate(tasks: &mut [Task], config: &Config, root: &Path) {
    for task in tasks {
        for (unit, count) in &mut task.estimate_counts {
            match resolve(unit, count, config, root) {
                Ok(matched) => {
                    count.problem = None;
                    task.estimate
                        .insert(unit.clone(), count.manual + matched as f64);
                }
                Err(problem) => {
                    count.problem = Some(problem);
                    task.estimate.remove(unit);
                }
            }
        }
    }
}

/// How many files the queries of `count` match inside `unit`'s path set.
fn resolve(
    unit: &str,
    count: &CountEstimate,
    config: &Config,
    root: &Path,
) -> Result<usize, String> {
    let set = config.tasks.estimate.queries.get(unit).ok_or_else(|| {
        format!(
            "`{unit}` takes a plain number; name its path set under [tasks.estimate.queries] to count files"
        )
    })?;
    let dicts = config.compiled_dicts();
    let admitted = path_filter(set, dicts)?;
    let tree = tree(root)?;
    let mut union = BTreeSet::new();
    for query in &count.queries {
        let key = (root.to_path_buf(), set.clone(), query.clone());
        let matched = matches(key, &tree, &admitted, dicts)?;
        union.extend(matched.iter().cloned());
    }
    Ok(union.len())
}

/// `glob` compiled as a one-entry path set.
fn path_filter(glob: &str, dicts: &Dicts) -> Result<PathFilter, String> {
    PathFilter::resolve(&[glob.to_owned()], dicts).map_err(|err| match err {
        PathSetError::Unknown { name, .. } => format!("no path set is named `@{name}`"),
        PathSetError::Invalid(message) => message,
    })
}

/// The files git lists in one working tree.
#[derive(Debug)]
struct Tree {
    /// The repository root the paths are relative to.
    top: PathBuf,
    /// Tracked and untracked-but-not-ignored files under the project root.
    files: Vec<String>,
}

/// Every working tree listed so far, keyed by project root.
static TREES: Mutex<BTreeMap<PathBuf, Result<Arc<Tree>, String>>> = Mutex::new(BTreeMap::new());

/// Every query run so far, keyed by project root, unit path set and query.
type MatchKey = (PathBuf, String, Query);
type Matched = Result<Arc<[String]>, String>;
static MATCHES: Mutex<BTreeMap<MatchKey, Matched>> = Mutex::new(BTreeMap::new());

/// The files git lists under `root`, read once per process.
fn tree(root: &Path) -> Result<Arc<Tree>, String> {
    let mut trees = TREES.lock().unwrap_or_else(PoisonError::into_inner);
    trees
        .entry(root.to_path_buf())
        .or_insert_with(|| list(root).map(Arc::new))
        .clone()
}

/// Ask git for `root`'s repository root and listed files.
fn list(root: &Path) -> Result<Tree, String> {
    let fail = |reason: String| format!("can't list files for a query: {reason}");
    let top = git(root, &["rev-parse", "--show-toplevel"]).map_err(fail)?;
    let listed = git(
        root,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--full-name",
        ],
    )
    .map_err(fail)?;
    let files: BTreeSet<String> = listed
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect();
    Ok(Tree {
        top: PathBuf::from(top.trim_end_matches(['\n', '\r'])),
        files: files.into_iter().collect(),
    })
}

/// The files the query in `key` matches inside `admitted`, run once per key.
fn matches(key: MatchKey, tree: &Tree, admitted: &PathFilter, dicts: &Dicts) -> Matched {
    let mut memo = MATCHES.lock().unwrap_or_else(PoisonError::into_inner);
    memo.entry(key)
        .or_insert_with_key(|(_, _, query)| run(tree, admitted, query, dicts).map(Arc::from))
        .clone()
}

/// Run `query` over the tree's files that `admitted` lets in.
fn run(
    tree: &Tree,
    admitted: &PathFilter,
    query: &Query,
    dicts: &Dicts,
) -> Result<Vec<String>, String> {
    let label = query.label();
    let (scope, regex) = match query {
        Query::Grep { regex, within } => {
            let scope = within
                .as_deref()
                .map(|glob| path_filter(glob, dicts))
                .transpose();
            let compiled = regex::bytes::Regex::new(regex)
                .map_err(|err| format!("{label} isn't a regex: {}", last_line(&err.to_string())))?;
            (scope, Some(compiled))
        }
        Query::Glob { pattern } => (path_filter(pattern, dicts).map(Some), None),
    };
    let scope = scope.map_err(|err| format!("{label}: {err}"))?;
    let candidates = tree.files.iter().filter(|path| {
        admitted.admits(path) && scope.as_ref().is_none_or(|scope| scope.admits(path))
    });
    // A tracked file deleted from the working tree is still listed; it
    // matches nothing.
    Ok(candidates
        .filter(|path| {
            let file = tree.top.join(path.as_str());
            regex.as_ref().map_or_else(
                || file.is_file(),
                |regex| std::fs::read(&file).is_ok_and(|bytes| regex.is_match(&bytes)),
            )
        })
        .cloned()
        .collect())
}

/// The last line of a `regex` error, which names what went wrong.
fn last_line(text: &str) -> String {
    text.lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(text)
        .trim()
        .to_owned()
}

/// Config-load problems in `[tasks.estimate.queries]`: a unit outside a
/// declared `units` list, or a path set that doesn't resolve.
pub(crate) fn validate_config(config: &Config) -> Vec<String> {
    let estimate = &config.tasks.estimate;
    let mut problems = Vec::new();
    for (unit, set) in &estimate.queries {
        let at = format!("tasks.estimate.queries.{unit}");
        if !estimate.units.is_empty() && !estimate.units.contains(unit) {
            problems.push(format!("{at}: `{unit}` is not an allowed estimate unit"));
        }
        if let Err(problem) = path_filter(set, config.compiled_dicts()) {
            problems.push(format!("{at}: {problem}"));
        }
    }
    problems
}

#[cfg(test)]
#[path = "estimate_queries_tests.rs"]
mod tests;
