//! Consumer-blind orchestration for retiring Typst artifacts.
//!
//! The shared cleanup skeleton across file-based trackers: refuse a dirty tree,
//! mint a `swh:1:rev` tombstone per retired file ([`swhid_mint`]), rewrite inbound
//! `link("…")` references in survivors ([`typst_edit`]) onto those tombstones, and
//! delete the retired files — without committing, so the operator reviews and
//! commits.
//!
//! Atomic by construction: [`plan`] mints every tombstone and computes every edit
//! in memory; not a byte is written until [`apply`] runs on a complete plan. A
//! wrong-shaped file or an unrecoverable commit aborts the run with no changes.
//!
//! The crate knows nothing about what an artifact *is*. The consumer supplies the
//! policy via [`Policy`]: which `link()` URLs name artifacts ([`Policy::link_identity`]),
//! and how a retired artifact's tombstone link reads ([`Policy::tombstone`]). It
//! also partitions its artifacts into the retired ([`Artifact`]s passed as
//! `eligible`) and the survivors to scan.

mod git;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub use git::{is_clean, toplevel};
// Re-exported because it appears in `Policy::tombstone`'s signature: consumers
// implementing the trait need to name the minted history pointer's type.
pub use swhid_mint::Swhid;

/// An artifact on disk: a file cleanup may retire, or scan for inbound links.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    /// Absolute path to the file.
    pub abs: PathBuf,
    /// Display path (root- or repository-relative), used in reports.
    pub rel: PathBuf,
    /// Identity a `link()` target resolves to (e.g. the file stem).
    pub id: String,
}

/// Consumer policy: how `link()` URLs map to artifact identities, and how a
/// retired artifact's tombstone link reads. Both methods are consumer-supplied —
/// the crate bakes in no identity scheme or label format.
pub trait Policy {
    /// Map a `link("…")` URL to the artifact identity it targets, or `None` when
    /// the URL is not an inbound artifact edge — an external URL, or an existing
    /// tombstone that must be left untouched.
    fn link_identity(&self, url: &str) -> Option<String>;

    /// Replacement text for a retired artifact's inbound link argument: the
    /// quoted SWHID tombstone URL, optionally carrying a synthesized label. Called
    /// once per retired artifact; `swhid` is its freshly minted history pointer.
    fn tombstone(&self, retired: &Artifact, swhid: &Swhid) -> String;
}

/// A file the plan will delete: its absolute path and display path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deletion {
    /// Absolute path to delete.
    pub abs: PathBuf,
    /// Display path for reports.
    pub rel: PathBuf,
}

/// A surviving file the plan will rewrite, with its already-spliced contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rewrite {
    /// Absolute path to overwrite.
    pub abs: PathBuf,
    /// Display path for reports.
    pub rel: PathBuf,
    /// The rewritten file contents.
    pub contents: String,
    /// How many links were retargeted in this file.
    pub links: usize,
}

/// The full, computed-in-memory cleanup. Empty when nothing is eligible.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Files to delete.
    pub deletions: Vec<Deletion>,
    /// Surviving files to rewrite.
    pub rewrites: Vec<Rewrite>,
}

impl Plan {
    /// Whether the plan deletes nothing (and so rewrites nothing).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.deletions.is_empty()
    }
}

/// Why a cleanup plan or apply failed.
#[derive(Debug)]
pub enum Error {
    /// An eligible file lies outside the git repository root.
    OutsideRepo(PathBuf),
    /// Minting the tombstone for a file failed (untracked, dirty, or git error).
    Mint(PathBuf, swhid_mint::git::Error),
    /// Reading a surviving file failed.
    Read(PathBuf, std::io::Error),
    /// Writing or deleting a file during apply failed.
    Write(PathBuf, std::io::Error),
    /// Splicing a surviving file's links failed (overlap, bad span).
    Rewrite(PathBuf, typst_edit::EditError),
    /// A repository or status query failed.
    Git(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutsideRepo(path) => {
                write!(f, "{} is outside the git repository", path.display())
            }
            Self::Mint(path, err) => write!(f, "mint tombstone for {}: {err}", path.display()),
            Self::Read(path, err) => write!(f, "read {}: {err}", path.display()),
            Self::Write(path, err) => write!(f, "write {}: {err}", path.display()),
            Self::Rewrite(path, err) => write!(f, "rewrite links in {}: {err}", path.display()),
            Self::Git(msg) => write!(f, "git: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

/// Compute the whole cleanup without touching disk.
///
/// Mints a tombstone for every `eligible` artifact (failing the run if any file
/// is untracked or its working bytes diverge from git) and splices every
/// `survivor`'s inbound links onto those tombstones. `git_root` is the repository
/// root every eligible path must lie within. Returns an empty [`Plan`] when
/// `eligible` is empty.
///
/// # Errors
/// [`Error`] on a file outside the repository, a mint failure, a survivor read
/// failure, or a rejected splice — before any byte is written.
pub fn plan<P: Policy>(
    eligible: &[Artifact],
    survivors: &[Artifact],
    git_root: &Path,
    policy: &P,
) -> Result<Plan, Error> {
    if eligible.is_empty() {
        return Ok(Plan::default());
    }
    let mut tombstones: HashMap<String, String> = HashMap::new();
    let mut deletions = Vec::new();
    for art in eligible {
        let git_rel = art
            .abs
            .strip_prefix(git_root)
            .map_err(|_| Error::OutsideRepo(art.abs.clone()))?
            .to_string_lossy();
        let id = swhid_mint::git::mint_rev_path(git_root, &git_rel)
            .map_err(|err| Error::Mint(art.rel.clone(), err))?;
        tombstones.insert(art.id.clone(), policy.tombstone(art, &id));
        deletions.push(Deletion {
            abs: art.abs.clone(),
            rel: art.rel.clone(),
        });
    }

    let mut rewrites = Vec::new();
    for art in survivors {
        if let Some(rewrite) = plan_rewrite(art, policy, &tombstones)? {
            rewrites.push(rewrite);
        }
    }

    deletions.sort_by(|a, b| a.rel.cmp(&b.rel));
    rewrites.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(Plan {
        deletions,
        rewrites,
    })
}

/// Build the rewrite for one survivor, or `None` if it links to no retired
/// artifact. Reads the source, locates every `link("…")`, and retargets those the
/// policy resolves to a retired identity — leaving every other link untouched.
fn plan_rewrite<P: Policy>(
    art: &Artifact,
    policy: &P,
    tombstones: &HashMap<String, String>,
) -> Result<Option<Rewrite>, Error> {
    let source =
        std::fs::read_to_string(&art.abs).map_err(|err| Error::Read(art.rel.clone(), err))?;
    let mut edits = Vec::new();
    for target in typst_edit::find_link_targets(&source) {
        let Some(id) = policy.link_identity(&target.url) else {
            continue;
        };
        if let Some(replacement) = tombstones.get(&id) {
            edits.push(typst_edit::Edit::new(
                target.range.clone(),
                replacement.clone(),
            ));
        }
    }
    if edits.is_empty() {
        return Ok(None);
    }
    let links = edits.len();
    let contents =
        typst_edit::apply(&source, edits).map_err(|err| Error::Rewrite(art.rel.clone(), err))?;
    Ok(Some(Rewrite {
        abs: art.abs.clone(),
        rel: art.rel.clone(),
        contents,
        links,
    }))
}

/// Write every rewrite, then delete every retired file. Call only on a complete,
/// validated [`Plan`] from [`plan`]; does not commit.
///
/// # Errors
/// [`Error::Write`] on the first file write or deletion that fails.
pub fn apply(plan: &Plan) -> Result<(), Error> {
    for rewrite in &plan.rewrites {
        std::fs::write(&rewrite.abs, &rewrite.contents)
            .map_err(|err| Error::Write(rewrite.rel.clone(), err))?;
    }
    for deletion in &plan.deletions {
        std::fs::remove_file(&deletion.abs)
            .map_err(|err| Error::Write(deletion.rel.clone(), err))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
