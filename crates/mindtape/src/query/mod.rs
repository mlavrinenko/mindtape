//! The shared file-query engine: one query value, its parse, compile and
//! match, plus the per-process caches behind them.
//!
//! Each [`Query`] names files git lists in the working tree: tracked ones, and
//! untracked ones `.gitignore` doesn't hide. Matching is native, `regex` over
//! file bytes and `globset` over paths, and never runs a shell. Paths are
//! relative to the repository root, as `changed()` reads them. Each project
//! root lists its files once per process, and each distinct query runs once.
//!
//! [`crate::estimate_queries`] is the first consumer: a `count(...)` unit
//! resolves its queries here. A later collector reuses the same engine.
//!
//! # Files
//!
//! - `tests.rs`: query parsing, labels and the engine's match rules.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use ast_grep_core::Pattern;
use ast_grep_language::LanguageExt;
use typst_harvest::HVal;

use crate::config::dicts::Dicts;
use crate::git::state::{PathFilter, PathSetError, git};

/// One file query inside `count(...)`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Query {
    /// `grep(regex, glob)`: the files whose bytes match `regex`, narrowed
    /// to `glob` when it is set.
    Grep {
        /// The pattern, in `regex` crate syntax.
        regex: String,
        /// A glob or `@name` path set the file must also fall in.
        glob: Option<String>,
    },
    /// `glob(pattern)`: the files whose path matches `pattern`.
    Glob {
        /// A glob or `@name` path set.
        pattern: String,
    },
    /// `ast_grep(language, pattern, glob)`: the files whose Rust source
    /// matches `pattern`, narrowed to `glob` when it is set.
    AstGrep {
        /// The language name, `rust` or its alias `rs`.
        language: String,
        /// The pattern, in ast-grep syntax.
        pattern: String,
        /// A glob or `@name` path set the file must also fall in.
        glob: Option<String>,
    },
}

impl Query {
    /// The query as `mt` names it, such as ``grep `parse(` in `src/**` ``.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Grep { regex, glob: None } => format!("grep `{regex}`"),
            Self::Grep {
                regex,
                glob: Some(glob),
            } => format!("grep `{regex}` in `{glob}`"),
            Self::Glob { pattern } => format!("glob `{pattern}`"),
            Self::AstGrep {
                language,
                pattern,
                glob: None,
            } => format!("ast-grep {language} `{pattern}`"),
            Self::AstGrep {
                language,
                pattern,
                glob: Some(glob),
            } => format!("ast-grep {language} `{pattern}` in `{glob}`"),
        }
    }
}

/// Read one query dict, or `None` when it is no query.
pub(crate) fn parse(value: &HVal) -> Option<Query> {
    let text = |key: &str| value.get(key).and_then(HVal::as_str).map(str::to_owned);
    match value.get("query").and_then(HVal::as_str)? {
        "grep" => Some(Query::Grep {
            regex: text("regex")?,
            glob: text("glob"),
        }),
        "glob" => Some(Query::Glob {
            pattern: text("pattern")?,
        }),
        "ast-grep" => Some(Query::AstGrep {
            language: text("language")?,
            pattern: text("pattern")?,
            glob: text("glob"),
        }),
        _ => None,
    }
}

/// `glob` compiled as a one-entry path set.
pub(crate) fn path_filter(glob: &str, dicts: &Dicts) -> Result<PathFilter, String> {
    PathFilter::resolve(&[glob.to_owned()], dicts).map_err(|err| match err {
        PathSetError::Unknown { name, .. } => format!("no path set is named `@{name}`"),
        PathSetError::Invalid(message) => message,
    })
}

/// The files the query matches inside `admitted`, under one process cache key
/// shared by every consumer of `root`'s `scope`.
pub(crate) fn matches(
    root: &Path,
    scope: &str,
    query: &Query,
    admitted: &PathFilter,
    dicts: &Dicts,
) -> Result<Arc<[String]>, String> {
    let tree = tree(root)?;
    let key = (root.to_path_buf(), scope.to_owned(), query.clone());
    let mut memo = MATCHES.lock().unwrap_or_else(PoisonError::into_inner);
    memo.entry(key)
        .or_insert_with(|| run(&tree, admitted, query, dicts).map(Arc::from))
        .clone()
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

/// A query compiled for this run.
enum Compiled {
    /// A regex over file bytes.
    Grep(regex::bytes::Regex),
    /// The path alone.
    Paths,
    /// An ast-grep pattern over Rust source.
    AstGrep(Pattern),
}

/// Run `query` over the tree's files that `admitted` lets in.
fn run(
    tree: &Tree,
    admitted: &PathFilter,
    query: &Query,
    dicts: &Dicts,
) -> Result<Vec<String>, String> {
    let label = query.label();
    let scope = match query {
        Query::Grep { glob, .. } | Query::AstGrep { glob, .. } => glob.as_deref(),
        Query::Glob { pattern } => Some(pattern.as_str()),
    };
    let scope = scope
        .map(|glob| path_filter(glob, dicts))
        .transpose()
        .map_err(|err| format!("{label}: {err}"))?;
    let compiled = compile(query, &label)?;
    let candidates = tree.files.iter().filter(|path| {
        admitted.admits(path) && scope.as_ref().is_none_or(|scope| scope.admits(path))
    });
    Ok(candidates
        .filter(|path| matches_file(tree, path, &compiled))
        .cloned()
        .collect())
}

/// Compile `query`, turning a bad regex, pattern or language into a problem.
fn compile(query: &Query, label: &str) -> Result<Compiled, String> {
    match query {
        Query::Grep { regex, .. } => regex::bytes::Regex::new(regex)
            .map(Compiled::Grep)
            .map_err(|err| format!("{label} isn't a regex: {}", last_line(&err.to_string()))),
        Query::Glob { .. } => Ok(Compiled::Paths),
        Query::AstGrep {
            language, pattern, ..
        } => {
            let rust = rust_language(language)?;
            let compiled = Pattern::try_new(pattern, rust).map_err(|err| {
                format!("{label} isn't a pattern: {}", last_line(&err.to_string()))
            })?;
            if compiled.has_error() {
                return Err(format!(
                    "{label} isn't a pattern: the grammar reports an error"
                ));
            }
            Ok(Compiled::AstGrep(compiled))
        }
    }
}

/// The one grammar this wave ships, named as a task may write it.
fn rust_language(language: &str) -> Result<ast_grep_language::Rust, String> {
    match language {
        "rust" | "rs" => Ok(ast_grep_language::Rust),
        other => Err(format!(
            "ast-grep language `{other}` isn't supported; supported: rust (alias rs)"
        )),
    }
}

/// Whether `path` matches the compiled query. A tracked file deleted from the
/// working tree is still listed; it matches nothing.
fn matches_file(tree: &Tree, path: &str, compiled: &Compiled) -> bool {
    let file = tree.top.join(path);
    match compiled {
        Compiled::Grep(regex) => std::fs::read(&file).is_ok_and(|bytes| regex.is_match(&bytes)),
        Compiled::Paths => file.is_file(),
        Compiled::AstGrep(pattern) => std::fs::read_to_string(&file).is_ok_and(|source| {
            ast_grep_language::Rust
                .ast_grep(source)
                .root()
                .find(pattern)
                .is_some()
        }),
    }
}

/// The last line of a `regex` error, which names what went wrong, without
/// its `error: ` label.
fn last_line(text: &str) -> String {
    let line = text
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(text)
        .trim();
    line.strip_prefix("error: ").unwrap_or(line).to_owned()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
