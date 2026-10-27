//! Which config commands the user has allowed to run in a repository.
//!
//! A clone's `.mindtape/config.toml` can name shell lines that `mt` runs: a
//! `[[tasks.waits]]` checker, which read commands run, an `[[on.flip]]` `run`
//! action, which `mt flip` runs, and the `[go] editor` and `[add] editor`
//! programs, which `mt go` and `mt add` spawn. None of them runs until the
//! user reads them and runs `mt allow`, which records a digest of every such
//! line ([`commands`]) for the repository. An editor the user's own
//! `<config home>/mindtape/defaults.toml` sets is the user's, and needs no
//! approval ([`UserCommands`]); [`runnable_editor`] decides. A changed line changes the digest, so it
//! asks again. A caller that skips a command for want of approval records it
//! with [`hold`] under its project root, and [`held_back`] lists them for the
//! one warning per process.
//!
//! The record lives outside the clone, under
//! `<data home>/mindtape/allowed/` ([`crate::data_home`]), so no file a repo
//! ships can approve its own commands. It is keyed by the repository's
//! `git rev-parse --git-common-dir`, or the project root outside git, so every
//! worktree of one repository shares one approval.
//!
//! A record file holds the digest on its first line, then the key and each
//! command as `#` comments for a person who opens it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use sha1::{Digest, Sha1};

use crate::config::Config;
use crate::config::layers::{Layer, LayerSource};
use crate::fs_atomic::write_atomic;

/// Every command line the config asks `mt` to run, sorted and deduplicated:
/// what [`digest`] covers and what `mt allow` lists. That is each
/// `[[tasks.waits]]` `run` line, each `[[on.flip]]` action's `run` line, and
/// the `[go] editor` and `[add] editor` commands unless the user's own layer
/// set them. Another kind of config command joins by adding its lines here,
/// and every approval then asks again.
#[must_use]
pub fn commands(config: &Config) -> Vec<String> {
    let checkers = config.tasks.waits.iter().map(|checker| checker.run.clone());
    let actions = config
        .on
        .flip
        .iter()
        .flat_map(|hook| &hook.actions)
        .filter_map(|action| action.run.clone());
    let editors = [&config.go.editor, &config.add.editor]
        .into_iter()
        .filter_map(|editor| untrusted_editor(config, editor.as_deref()));
    let mut commands: Vec<String> = checkers.chain(actions).chain(editors).collect();
    commands.sort();
    commands.dedup();
    commands
}

/// The editor commands the user's own config layer sets, which run without
/// approval. Derived at load by [`crate::config::layers`], never declared,
/// never equal-compared, like [`crate::rules::RulesRoot`]: it is empty for a
/// [`Config`] built through [`crate::config::parse_config`] alone, so every
/// editor there is a project's. Only `<config home>/mindtape/defaults.toml`
/// counts. An ancestor's `.mindtape/defaults.toml` can sit inside a clone, so
/// it is no more the user's than the project's own file.
#[derive(Debug, Clone, Default)]
pub(crate) struct UserCommands(BTreeSet<String>);

impl PartialEq for UserCommands {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl UserCommands {
    /// The trimmed `[go] editor` and `[add] editor` of every global layer in
    /// `layers`. A layer that is not TOML sets none: the merge reports it.
    pub(crate) fn from_layers(layers: &[Layer]) -> Self {
        let editors = layers
            .iter()
            .filter(|layer| matches!(layer.source, LayerSource::Xdg(_)))
            .filter_map(|layer| layer.text.parse::<toml::Table>().ok())
            .flat_map(|table| [editor_of(&table, "go"), editor_of(&table, "add")])
            .flatten();
        Self(editors.collect())
    }
}

/// `table`'s `[<section>] editor`, trimmed, when it is a non-blank string.
fn editor_of(table: &toml::Table, section: &str) -> Option<String> {
    let editor = table.get(section)?.get("editor")?.as_str()?.trim();
    (!editor.is_empty()).then(|| editor.to_owned())
}

/// `editor` trimmed, unless it is blank or the user's own layer set it.
fn untrusted_editor(config: &Config, editor: Option<&str>) -> Option<String> {
    let editor = editor?.trim();
    (!editor.is_empty() && !config.user_commands.0.contains(editor)).then(|| editor.to_owned())
}

/// The editor command a caller may spawn: `editor`, a `[go] editor` or
/// `[add] editor` of `config`, when the user's own layer set it or the user
/// allowed the config's commands for `root`'s repository. Otherwise `None`,
/// and the editor is held back for the warning that asks for `mt allow`, so
/// the caller falls back to `$VISUAL`. A blank or absent editor is `None` and
/// holds nothing.
#[must_use]
pub fn runnable_editor(root: &Path, config: &Config, editor: Option<&str>) -> Option<String> {
    runnable_editor_in(crate::data_home().as_deref(), root, config, editor)
}

/// [`runnable_editor`] against the data home `home`.
fn runnable_editor_in(
    home: Option<&Path>,
    root: &Path,
    config: &Config,
    editor: Option<&str>,
) -> Option<String> {
    let editor = editor?.trim();
    if editor.is_empty() {
        return None;
    }
    if untrusted_editor(config, Some(editor)).is_none() || allowed_in(home, root, &commands(config))
    {
        return Some(editor.to_owned());
    }
    hold(root, &[editor.to_owned()]);
    None
}

/// Every command this process held back because the user has not allowed it,
/// by the root of the project whose config declares it.
static HELD: Mutex<BTreeMap<PathBuf, BTreeSet<String>>> = Mutex::new(BTreeMap::new());

/// Record `commands` of the project at `root` as held back: a caller found
/// them not allowed and ran none of them.
pub fn hold(root: &Path, commands: &[String]) {
    let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    held.entry(root.to_path_buf())
        .or_default()
        .extend(commands.iter().cloned());
}

/// The commands this process would have run but held back, from every
/// project, sorted once each: what the warning that asks for `mt allow` lists.
/// Empty when nothing was held back.
#[must_use]
pub fn held_back() -> Vec<String> {
    let held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    let all: BTreeSet<&String> = held.values().flatten().collect();
    all.into_iter().cloned().collect()
}

/// Take the commands held back for the project at `root`, sorted, so a caller
/// that warns for one member of a set names that project, and [`held_back`]
/// no longer lists them.
#[must_use]
pub fn take_held(root: &Path) -> Vec<String> {
    let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    held.remove(root)
        .map(|commands| commands.into_iter().collect())
        .unwrap_or_default()
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
    crate::canonical(&key).unwrap_or(key)
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
