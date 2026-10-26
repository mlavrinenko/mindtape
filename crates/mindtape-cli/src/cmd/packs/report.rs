//! What `mt packs update` prints under a moved source: each pack and file
//! the move changes, and every new command line in full, so the lock's diff
//! is not the only record a reviewer gets.
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

use mindtape::config::pack_sources::{self, DiffError, LockedSource, TreeDiff};

use crate::diag::{self, Diag};

/// Print what moving `name` from the `old` lock entry to the tree at `new`
/// changes. When the old tree can't be had or read, warn and print nothing.
pub(super) fn print_diff(
    root: &Path,
    name: &str,
    old: Option<&LockedSource>,
    new: &Path,
    cache: &Path,
) {
    let diff = old
        .map(|old| old_tree(root, old, cache))
        .transpose()
        .and_then(|old| pack_sources::diff_trees(old.as_deref(), new));
    match diff {
        Ok(diff) => print!("{}", render(name, &diff)),
        Err(error) => diag::emit(
            &Diag::warning("can't diff pack source")
                .value(name)
                .cause(&error),
        ),
    }
}

/// The tree `old` pins, fetched by its commit when the cache lacks it.
fn old_tree(root: &Path, old: &LockedSource, cache: &Path) -> Result<PathBuf, DiffError> {
    let tree = cache.join(&old.sha256);
    if !tree.is_dir() {
        let git = pack_sources::locate(&old.git, root);
        pack_sources::fetch(&git, &old.commit, &old.commit, Some(&old.sha256), cache)
            .map_err(DiffError::Read)?;
    }
    Ok(tree)
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
