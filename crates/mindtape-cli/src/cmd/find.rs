//! `mt find TERM…` is sugar over `contains()`, sharing `ls`'s code path.
//!
//! Pure desugaring: `mt find login bug` becomes the filter expression
//! `contains("login") && contains("bug")`, which goes to
//! [`crate::cmd::ls::run_with`] exactly as a manual `mt ls '...'` filter would.
//! There is no second matcher and no field vocabulary of its own. `find` grows
//! no options past its terms. The moment it did, it would become a second query
//! engine that drifts from the filter language, the failure
//! `crate::model::fields` (`mindtape::model::fields`) exists to end (see
//! `truth/tasks/2026-08/mt-find-as-sugar-over-contains.typ`).

use crate::diag::Result;

use crate::cmd::ls::{LsArgs, LsFlags};
use crate::util;

/// Arguments for `find`. Every flag is [`LsFlags`], flattened, the same struct
/// `ls` uses, so a flag can never exist on one and not the other. No `--in`:
/// field targeting stays in the filter language
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
/// Returns an error if the project does not open or the desugared filter does
/// not pass (not expected for this shape, but propagated all the same).
pub fn run(args: &FindArgs) -> Result<()> {
    if let Some(held) = crate::project_set::held_set() {
        return crate::cmd::ls::set::run(&ls_args(args), held);
    }
    run_with(args, util::open_warned()?)
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

/// `["login", "bug"]` → `contains("login") && contains("bug")`. The function
/// quotes each term as a filter-language string literal ([`quote`]) so it
/// reaches `contains()` as one needle regardless of what characters it holds.
fn desugar(terms: &[String]) -> String {
    terms
        .iter()
        .map(|term| format!("contains({})", quote(term)))
        .collect::<Vec<_>>()
        .join(" && ")
}

/// Quote `term` as a filter-language string literal. Backslash escapes come
/// first, so an already-escaped sequence in the term is not double-escaped,
/// then the closing-quote character. Those are the only two escapes the filter
/// language's `evalexpr`-based lexer accepts
/// (`crates/mindtape/src/filter/mod.rs`).
fn quote(term: &str) -> String {
    format!("\"{}\"", term.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
#[path = "find_tests.rs"]
mod tests;
