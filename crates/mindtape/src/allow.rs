//! Which config commands the user has allowed to run in a repository.
//!
//! A clone's `.mindtape/config.toml` can name shell lines that `mt` runs: a
//! `[[tasks.waits]]` checker, which read commands run, and an `[[on.flip]]`
//! `run` action, which `mt flip` runs. None of them runs until the user reads
//! them and runs `mt allow`, which records a digest of every such line
//! ([`commands`]) for the repository. A changed line changes the digest, so it
//! asks again. A caller that skips a command for want of approval records it
//! with [`hold`], and [`held_back`] lists them for the one warning per process.
//!
//! The record lives outside the clone, under
//! `<data home>/mindtape/allowed/` ([`crate::data_home`]), so no file a repo
//! ships can approve its own commands. It is keyed by the repository's
//! `git rev-parse --git-common-dir`, or the project root outside git, so every
//! worktree of one repository shares one approval.
//!
//! A record file holds the digest on its first line, then the key and each
//! command as `#` comments for a person who opens it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use sha1::{Digest, Sha1};

use crate::config::Config;
use crate::fs_atomic::write_atomic;

/// Every command line the config asks `mt` to run, sorted and deduplicated:
/// what [`digest`] covers and what `mt allow` lists. That is each
/// `[[tasks.waits]]` `run` line and each `[[on.flip]]` action's `run` line.
/// Another kind of config command joins by adding its lines here, and every
/// approval then asks again.
#[must_use]
pub fn commands(config: &Config) -> Vec<String> {
    let checkers = config.tasks.waits.iter().map(|checker| checker.run.clone());
    let actions = config
        .on
        .flip
        .iter()
        .flat_map(|hook| &hook.actions)
        .filter_map(|action| action.run.clone());
    let mut commands: Vec<String> = checkers.chain(actions).collect();
    commands.sort();
    commands.dedup();
    commands
}

/// Every command this process held back because the user has not allowed it.
static HELD: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

/// Record `commands` as held back: a caller found them not allowed and ran
/// none of them.
pub fn hold(commands: &[String]) {
    let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    held.extend(commands.iter().cloned());
}

/// The commands this process would have run but held back, sorted: what the
/// warning that asks for `mt allow` lists. Empty when nothing was held back.
#[must_use]
pub fn held_back() -> Vec<String> {
    let held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    held.iter().cloned().collect()
}

/// The digest an approval records for `commands`: each line, NUL-terminated,
/// through SHA-1, as lowercase hex.
#[must_use]
pub fn digest(commands: &[String]) -> String {
    let mut hasher = Sha1::new();
    for command in commands {
        hasher.update(command.as_bytes());
        hasher.update([0]);
    }
    hex(&hasher.finalize())
}

/// What one approval belongs to: the repository's common git directory, or
/// `root` itself outside git, made absolute.
#[must_use]
pub fn repo_key(root: &Path) -> PathBuf {
    let common = crate::git::state::git(root, &["rev-parse", "--git-common-dir"])
        .ok()
        .map(|out| out.trim().to_owned())
        .filter(|out| !out.is_empty());
    let key = common.map_or_else(|| root.to_path_buf(), |dir| root.join(dir));
    key.canonicalize().unwrap_or(key)
}

/// The record file for `key` under `data_home`.
#[must_use]
pub fn record_path(data_home: &Path, key: &Path) -> PathBuf {
    let name = hex(&Sha1::digest(key.as_os_str().as_encoded_bytes()));
    data_home.join("mindtape").join("allowed").join(name)
}

/// Whether the user allowed `commands` for the repository holding `root`.
/// An empty list needs no approval. With no data home there is nowhere to
/// record one, so nothing is allowed.
#[must_use]
pub fn is_allowed(root: &Path, commands: &[String]) -> bool {
    allowed_in(crate::data_home().as_deref(), root, commands)
}

/// [`is_allowed`] against the data home `home`.
fn allowed_in(home: Option<&Path>, root: &Path, commands: &[String]) -> bool {
    if commands.is_empty() {
        return true;
    }
    let Some(home) = home else {
        return false;
    };
    let path = record_path(home, &repo_key(root));
    std::fs::read_to_string(path)
        .is_ok_and(|text| text.lines().next() == Some(digest(commands).as_str()))
}

/// Why [`record`] could not write the approval.
#[derive(Debug, thiserror::Error)]
pub enum AllowError {
    /// `$MINDTAPE_DATA_HOME` is set but empty, or the platform has no data
    /// directory.
    #[error("no data directory to record the approval in")]
    NoDataHome,
    /// The record file could not be written.
    #[error("can't write {}: {source}", path.display())]
    Write {
        /// The record file.
        path: PathBuf,
        /// The OS's reason.
        source: std::io::Error,
    },
}

/// Record `commands` as allowed for the repository holding `root`, replacing
/// any earlier approval, and return the record file.
///
/// # Errors
/// [`AllowError::NoDataHome`] when no data directory applies, and
/// [`AllowError::Write`] when the file cannot be written.
pub fn record(root: &Path, commands: &[String]) -> Result<PathBuf, AllowError> {
    record_in(crate::data_home().as_deref(), root, commands)
}

/// [`record`] under the data home `home`.
fn record_in(home: Option<&Path>, root: &Path, commands: &[String]) -> Result<PathBuf, AllowError> {
    let home = home.ok_or(AllowError::NoDataHome)?;
    let key = repo_key(root);
    let path = record_path(home, &key);
    let mut text = format!("{}\n# {}\n", digest(commands), key.display());
    for command in commands {
        text.push_str("# ");
        text.push_str(command);
        text.push('\n');
    }
    let fail = |source| AllowError::Write {
        path: path.clone(),
        source,
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(fail)?;
    }
    write_atomic(&path, text.as_bytes()).map_err(fail)?;
    Ok(path)
}

/// `bytes` as lowercase hex.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
#[path = "allow_tests.rs"]
mod tests;
