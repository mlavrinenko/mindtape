//! What `mt packs update` prints under a moved source: each pack and file
//! the move changes, and every new command line in full, so the lock's diff
//! is not the only record a reviewer gets. The report waits in a [`Report`]
//! until the lock is written.
//!
//! ```text
//! acme v1: 3f2c1a9 → 8d01e44
//!   ~ acme:commits
//!     + on.flip.gate
//!         run "just check"
//!   + acme:ops
//!   ~ file team/check.typ
//! ```

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use mindtape::config::pack_sources::{self, FetchError, LockedSource, TreeDiff};

use crate::diag::{self, Diag};

/// What `mt packs update` prints, and the warnings it emits, held until the
/// lock is written.
#[derive(Default)]
pub(super) struct Report {
    out: String,
    warnings: Vec<Diag>,
}

impl Report {
    /// Add one line.
    pub(super) fn line(&mut self, line: &str) {
        self.out.push_str(line);
        self.out.push('\n');
    }

    /// Print the lines, then emit the warnings.
    pub(super) fn print(self) {
        print!("{}", self.out);
        for warning in &self.warnings {
            diag::emit(warning);
        }
    }
}

/// A source `mt packs update` moved: its lock entry before, the repository
/// its `git` names now, and the tree it moved to.
pub(super) struct Moved<'a> {
    pub(super) name: &'a str,
    pub(super) old: Option<&'a LockedSource>,
    pub(super) git: &'a str,
    pub(super) tree: &'a Path,
}

impl Moved<'_> {
    /// Add what the move changes to `report`. When the old tree can't be
    /// had, warn and list the new tree as all added, so no new command goes
    /// unlisted.
    pub(super) fn report(&self, root: &Path, cache: &Path, report: &mut Report) {
        let old = match self.old.map(|old| self.old_tree(root, old, cache)) {
            Some(Err(error)) => {
                let warning = Diag::warning("can't fetch the old tree of pack source");
                report.warnings.push(warning.value(self.name).cause(&error));
                None
            }
            Some(Ok(tree)) => Some(tree),
            None => None,
        };
        match pack_sources::diff_trees(old.as_deref(), self.tree) {
            Ok(diff) => report.out.push_str(&render(self.name, &diff)),
            Err(error) => {
                let warning = Diag::warning("can't diff pack source");
                report.warnings.push(warning.value(self.name).cause(&error));
            }
        }
    }

    /// The tree `old` pins, fetched by its commit when the cache lacks it:
    /// from `old`'s own `git`, else from the source's `git` now, since a
    /// source may move hosts. The lock's hash checks either.
    fn old_tree(
        &self,
        root: &Path,
        old: &LockedSource,
        cache: &Path,
    ) -> Result<PathBuf, FetchError> {
        let tree = cache.join(&old.sha256);
        if tree.is_dir() {
            return Ok(tree);
        }
        let fetch = |git: &str| {
            pack_sources::fetch(git, &old.commit, &old.commit, Some(&old.sha256), cache)
        };
        let from_old = pack_sources::locate(&old.git, root).and_then(|git| fetch(&git));
        match from_old {
            Ok(_) => Ok(tree),
            Err(_) => fetch(self.git).map(|_| tree),
        }
    }
}

/// The report lines of `diff`, each pack named `<name>:<pack>`.
fn render(name: &str, diff: &TreeDiff) -> String {
    let mut out = String::new();
    for pack in &diff.packs {
        let _ = writeln!(out, "  {} {name}:{}", pack.change.sign(), pack.pack);
        for entry in &pack.entries {
            let _ = writeln!(out, "    {} {}", entry.change.sign(), entry.label);
            commands(&mut out, "        ", &entry.commands);
        }
        commands(&mut out, "      ", &pack.commands);
    }
    for (change, path) in &diff.files {
        let _ = writeln!(out, "  {} file {path}", change.sign());
    }
    out
}

/// One `run "<line>"` line per command, quoted so a control character shows.
fn commands(out: &mut String, indent: &str, lines: &[String]) {
    for line in lines {
        let _ = writeln!(out, "{indent}run {line:?}");
    }
}
