//! The ast-grep collector: run the shared file-query engine and publish one
//! matching path per file as `derived.<name>`.
//!
//! The query reuses [`crate::query`] — the same git-listed file walk, Rust
//! grammar and per-process caches an estimate's `ast_grep(...)` uses. No path
//! set narrows a collector, so every listed file is admitted unless the query
//! carries its own `glob`. A failed walk or a query the engine refuses is one
//! [`Problem`] naming the field, which [`crate::validate`] reports as a
//! `Collector` violation at `derived.<name>`.

use typst_harvest::HVal;

use crate::config::Config;
use crate::config::collect::AstGrepQuery;
use crate::git::state::PathFilter;
use crate::problem::Problem;
use crate::query::{self, Query};

/// The paths `query` matches under `config`'s project root, or one problem
/// naming the collector field.
pub(crate) fn values(
    name: &str,
    query: &AstGrepQuery,
    config: &Config,
) -> Result<Vec<HVal>, Problem> {
    let ast = &query.ast_grep;
    let question = Query::AstGrep {
        language: ast.language.clone(),
        pattern: ast.pattern.clone(),
        glob: ast.glob.clone(),
    };
    let files = query::matches(
        config.rules_root.as_path(),
        "",
        &question,
        &PathFilter::all(),
        config.compiled_dicts(),
    )
    .map_err(|err| fail(name, &err))?;
    Ok(files.iter().cloned().map(HVal::Str).collect())
}

/// One problem naming the derived field and why the engine refused.
fn fail(name: &str, message: &str) -> Problem {
    Problem::new(format!("derived.{name}: {message}"))
}
