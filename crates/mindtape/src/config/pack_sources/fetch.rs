//! Fetching one locked source into the cache: `git fetch` of its `ref` into
//! a temporary repository inside the cache root, the locked commit's files
//! written from the objects themselves, the tree hashed with
//! [`crate::config::pack_sources::hash`], and the result renamed to
//! `<cache root>/<sha256>/`.
//!
//! The files come from `git ls-tree` and `git cat-file --batch`, not a
//! checkout, so neither the machine's `core.autocrlf` nor the source's
//! `.gitattributes`, filters or hooks change a byte: every platform writes
//! and hashes the bytes the commit stores. A 64-digit commit is a SHA-256
//! repository's, so the scratch repository takes that object format.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::config::pack_sources::git::Git;
use crate::config::pack_sources::hash;
use crate::config::pack_sources::listing::parse_listing;
use crate::config::pack_sources::scratch::{STALE_AFTER, Scratch, settle, sweep};
use crate::config::quote;

/// Why a fetch failed.
#[derive(Debug, Error)]
pub enum FetchError {
    /// No `git` program could be started.
    #[error("git not found")]
    GitNotFound,
    /// A git command exited non-zero.
    #[error("git {step} failed: {message}")]
    Git {
        /// The git subcommand, such as `fetch`.
        step: &'static str,
        /// Git's own first line on stderr.
        message: String,
    },
    /// The locked commit is not hex of a commit id's length.
    #[error("not a commit id: {}", quote(commit))]
    BadCommit {
        /// The value given as the commit.
        commit: String,
    },
    /// The fetched `ref` doesn't reach the locked commit.
    #[error("commit {} is not on ref {}", quote(commit), quote(reference))]
    MissingCommit {
        /// The locked commit.
        commit: String,
        /// The ref that was fetched.
        reference: String,
    },
    /// The source holds an entry the cache can't hold the same way on every
    /// platform.
    #[error("pack source holds a {what}: {}", quote(path))]
    Unsupported {
        /// What the entry is, such as `symlink`.
        what: &'static str,
        /// The entry's path in the source.
        path: String,
    },
    /// The fetched tree's hash isn't the one expected.
    #[error(
        "pack source hash mismatch: expected {}, got {}",
        quote(expected),
        quote(actual)
    )]
    Mismatch {
        /// The hash the lock holds.
        expected: String,
        /// The hash of the fetched tree.
        actual: String,
    },
    /// The cache directory the hash names holds a tree of another hash.
    #[error("cache directory holds other files: {}", quote(&dir.display().to_string()))]
    Occupied {
        /// The directory.
        dir: PathBuf,
    },
    /// Reading or writing the cache failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl FetchError {
    /// What to try next, when there is an obvious next step.
    #[must_use]
    pub fn hint(&self) -> Option<&'static str> {
        match self {
            Self::GitNotFound => Some("install git and put it on PATH"),
            Self::MissingCommit { .. } => Some("run `mt packs update` to lock the ref anew"),
            Self::Occupied { .. } => Some("delete the directory: the cache is safe to delete"),
            Self::Git { .. }
            | Self::BadCommit { .. }
            | Self::Unsupported { .. }
            | Self::Mismatch { .. }
            | Self::Io(_) => None,
        }
    }

    pub(crate) fn unsupported(what: &'static str, path: &str) -> Self {
        Self::Unsupported {
            what,
            path: path.to_owned(),
        }
    }
}

/// Fetch `reference` of the repository `git`, write `commit`'s files to
/// `<cache_root>/<sha256>/`, and return the sha256. With `expected`, the
/// lock's hash, a tree of another hash fails before it reaches the cache.
///
/// # Errors
/// Returns [`FetchError`] when git is missing or fails, the commit is not
/// on the ref, the tree holds a symlink or a submodule, its hash isn't
/// `expected`, or the cache can't be written.
pub fn fetch(
    git: &str,
    reference: &str,
    commit: &str,
    expected: Option<&str>,
    cache_root: &Path,
) -> Result<String, FetchError> {
    let request = Fetch {
        program: OsStr::new("git"),
        git,
        reference,
        commit,
        expected,
    };
    request.run(cache_root)
}

/// Where git finds the source `git` that the project under `root` declares.
/// [`fetch`] runs git in the caller's directory, so a relative path is joined
/// to `root`. A URL, an scp-like `host:path` and an absolute path pass as
/// given.
#[must_use]
pub fn locate(git: &str, root: &Path) -> String {
    if is_remote(git) || Path::new(git).is_absolute() {
        return git.to_owned();
    }
    root.join(git).display().to_string()
}

/// Whether git reads `git` as a URL rather than a path: it holds `://`, or a
/// colon before any slash after more than one character, as `host:path`
/// does and a drive letter's `C:` doesn't.
fn is_remote(git: &str) -> bool {
    if git.contains("://") {
        return true;
    }
    git.split_once(':')
        .is_some_and(|(host, _)| host.len() > 1 && !host.contains(['/', '\\']))
}

/// One [`fetch`], with the git program it runs.
pub(super) struct Fetch<'a> {
    pub(super) program: &'a OsStr,
    pub(super) git: &'a str,
    pub(super) reference: &'a str,
    pub(super) commit: &'a str,
    pub(super) expected: Option<&'a str>,
}

impl Fetch<'_> {
    pub(super) fn run(&self, cache_root: &Path) -> Result<String, FetchError> {
        if !is_commit_id(self.commit) {
            return Err(FetchError::BadCommit {
                commit: self.commit.to_owned(),
            });
        }
        let git = Git::new(self.program)?;
        std::fs::create_dir_all(cache_root)?;
        sweep(cache_root, STALE_AFTER);
        let scratch = Scratch::new(cache_root)?;
        let repo = scratch.0.join("git");
        let tree = scratch.0.join("tree");
        let format = if self.commit.len() == 64 {
            "sha256"
        } else {
            "sha1"
        };
        let object_format = format!("--object-format={format}");
        let init = [
            os("init"),
            os("--quiet"),
            os("--bare"),
            os(&object_format),
            repo.as_os_str(),
        ];
        git.run(None, &init, "init")?;
        let fetch = [
            "fetch",
            "--quiet",
            "--no-tags",
            "--end-of-options",
            self.git,
            self.reference,
        ];
        git.run(Some(&repo), &fetch.map(os), "fetch")?;
        self.write_tree(&git, &repo, &tree)?;
        let actual = hash(&tree)?;
        if let Some(expected) = self.expected.filter(|expected| *expected != actual) {
            return Err(FetchError::Mismatch {
                expected: expected.to_owned(),
                actual,
            });
        }
        settle(&tree, &cache_root.join(&actual), &actual)?;
        Ok(actual)
    }

    /// Write every file of the commit under `tree`, from `repo`'s objects.
    fn write_tree(&self, git: &Git<'_>, repo: &Path, tree: &Path) -> Result<(), FetchError> {
        let spec = format!("{}^{{commit}}", self.commit);
        let exists = [os("cat-file"), os("-e"), os(&spec)];
        if git.run(Some(repo), &exists, "cat-file").is_err() {
            return Err(FetchError::MissingCommit {
                commit: self.commit.to_owned(),
                reference: self.reference.to_owned(),
            });
        }
        let list = ["ls-tree", "-r", "-z", self.commit].map(os);
        let blobs = parse_listing(&git.run(Some(repo), &list, "ls-tree")?)?;
        std::fs::create_dir(tree)?;
        git.write_blobs(repo, &blobs, tree)
    }
}

fn os(arg: &str) -> &OsStr {
    OsStr::new(arg)
}

/// A commit id is 40 or 64 lowercase hex digits.
fn is_commit_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .chars()
            .all(|ch| ch.is_ascii_digit() || matches!(ch, 'a'..='f'))
}

#[cfg(test)]
#[path = "fetch_tests.rs"]
mod tests;
