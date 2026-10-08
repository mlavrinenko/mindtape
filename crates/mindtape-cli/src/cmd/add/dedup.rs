//! `mt add --dedup`: collision policy on a destination stem that already
//! exists.
//!
//! [`resolve`] is fs-free — like [`mindtape::model::naming::disambiguate`] itself, it
//! takes an injected `exists` predicate rather than touching disk, so the
//! `add-suffix` exhaustion path (10,000 numeric-suffix attempts) is a fast
//! unit test rather than a directory full of real files. `crate::cmd::add::run`
//! supplies the real filesystem check; tests supply a fake.

use mindtape::{Config, model::naming};

use crate::diag::{Diag, Hint};

/// `--dedup` on `mt add`. A CLI-owned mirror of [`naming::DedupMode`]:
/// `mindtape` carries no clap dependency, so the flag's `clap::ValueEnum`
/// lives here and converts into the config-side enum rather than deriving it
/// there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum DedupArg {
    /// Stop without writing anything.
    Fail,
    /// Write nothing, print the existing task's path, and exit 0.
    Skip,
    /// Add `-2`, `-3` and so on to the name until it's free.
    AddSuffix,
}

impl From<DedupArg> for naming::DedupMode {
    fn from(value: DedupArg) -> Self {
        match value {
            DedupArg::Fail => Self::Fail,
            DedupArg::Skip => Self::Skip,
            DedupArg::AddSuffix => Self::AddSuffix,
        }
    }
}

/// The effective dedup mode for one `add` run: `--dedup` wins outright, else
/// [`naming::NamingConfig::effective_dedup`] — which is the single place that
/// picks a default between `[tasks.naming] dedup` and `stem`, so this
/// function never re-derives it from whether the stem happened to be
/// templated.
#[must_use]
pub fn effective_mode(flag: Option<DedupArg>, config: &Config) -> naming::DedupMode {
    flag.map_or_else(|| config.tasks.naming.effective_dedup(), Into::into)
}

/// What `dedup` decided about `stem`, given `exists`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Collision {
    /// `exists(stem)` was false: write at the requested stem.
    None,
    /// `dedup: skip` found `stem` taken; the caller writes nothing.
    Skipped,
    /// `dedup: add-suffix` minted this free stem instead.
    Suffixed(String),
}

/// Resolve `stem` against `dedup` and `exists`. `label` is the requested
/// stem's whole `<root>/<stem>.typ`-style display path, folded into the
/// `Fail` error only — `exists` itself never touches disk here (real disk
/// access is the caller's closure).
///
/// # Errors
/// `Fail` on a collision reports `label`; `AddSuffix` propagates
/// `naming::disambiguate`'s exhaustion error once every numeric suffix up to
/// `naming::MAX_DISAMBIGUATE_ATTEMPTS`-equivalent is reported taken.
pub fn resolve(
    dedup: naming::DedupMode,
    stem: &str,
    label: &str,
    exists: impl Fn(&str) -> bool,
) -> Result<Collision, Diag> {
    if !exists(stem) {
        return Ok(Collision::None);
    }
    match dedup {
        naming::DedupMode::Fail => Err(Diag::refused("task already exists")
            .value(label)
            .hint(Hint::new(
                "pass --dedup add-suffix to file it under a free name, or --dedup skip to keep the old one",
            ))),
        naming::DedupMode::Skip => Ok(Collision::Skipped),
        naming::DedupMode::AddSuffix => naming::disambiguate(stem, exists)
            .map(Collision::Suffixed)
            .map_err(|err| {
                Diag::failed("no free file name")
                    .value(stem)
                    .cause(&err)
                    .hint(Hint::new("pass --name"))
            }),
    }
}

#[cfg(test)]
#[path = "dedup_tests.rs"]
mod tests;
