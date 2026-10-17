//! `mt find TERM…` — sugar over `contains()`, sharing `ls`'s code path.
//!
//! Pure desugaring: `mt find login bug` becomes the filter expression
//! `contains("login") && contains("bug")`, which is handed to
//! [`crate::cmd::ls::run_with`] exactly as a manual `mt ls '...'` filter would be —
//! no second matcher, no field vocabulary of its own. `find` grows no options
//! past its terms; the moment it did, it would become a second query engine
//! that drifts from the filter language, the failure `crate::model::fields`
//! (`mindtape::model::fields`) was created to end (see
//! `truth/tasks/mt-find-as-sugar-over-contains.typ`).

use crate::diag::Result;

use crate::cmd::ls::{LsArgs, LsFlags};
use crate::util;

/// Arguments for `find`. Every flag is [`LsFlags`], flattened — the same
/// struct `ls` uses, so a flag can never exist on one and not the other. No
/// `--in`: field targeting stays in the filter language
/// (`mt ls 'contains("bug", title)'`).
#[derive(Debug, clap::Args)]
pub struct FindArgs {
    /// Words to search for.
    ///
    /// A task matches when it contains every word.

    #[arg(required = true, value_name = "TERM")]
    pub terms: Vec<String>,

    #[command(flatten)]
    pub flags: LsFlags,
}

/// Run `find`: desugar `terms` and hand the result to [`crate::cmd::ls::run_with`].
///
/// # Errors
/// Returns an error if the project cannot be opened or the desugared filter
/// is rejected (not expected for this shape, but propagated all the same).
pub fn run(args: &FindArgs) -> Result<()> {
    if let Some(held) = crate::project_set::held_set() {
        return crate::cmd::ls::set::run(&ls_args(args), held);
    }
    run_with(args, util::open()?)
}

pub(crate) fn run_with(args: &FindArgs, project: util::Project) -> Result<()> {
    crate::cmd::ls::run_with(&ls_args(args), project)
}

/// The `ls` run `args` stands for.
fn ls_args(args: &FindArgs) -> LsArgs {
    LsArgs {
        filter: Some(desugar(&args.terms)),
        flags: args.flags.clone(),
    }
}

/// `["login", "bug"]` → `contains("login") && contains("bug")`. Each term is
/// quoted as a filter-language string literal ([`quote`]) so it reaches
/// `contains()` as one needle regardless of what characters it holds.
fn desugar(terms: &[String]) -> String {
    terms
        .iter()
        .map(|term| format!("contains({})", quote(term)))
        .collect::<Vec<_>>()
        .join(" && ")
}

/// Quote `term` as a filter-language string literal: backslash escapes first
/// (so an already-escaped sequence in the term is not double-escaped), then
/// the closing-quote character — the only two escapes the filter language's
/// `evalexpr`-based lexer accepts (`crates/mindtape/src/filter/mod.rs`).
fn quote(term: &str) -> String {
    format!("\"{}\"", term.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
#[path = "find_tests.rs"]
mod tests;
