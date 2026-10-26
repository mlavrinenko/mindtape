//! What moving a source from one tree to another changes, compared as parsed
//! TOML, not text, so a reordered or reformatted pack reads as unchanged.
//!
//! Each pack at the top of a tree is diffed key by key. A table recurses,
//! and an array of tables is diffed element by element, each keyed by its
//! `name`, or by its position when it has none, so a rule or an `on.flip`
//! entry reads as one change. Any other value is one leaf. Every command a
//! new or changed entry carries, each `run` and `editor` line that
//! `mt allow` must approve, is listed in full. Every other file of the tree,
//! such as a template, a check module or a dictionary, is diffed by its
//! bytes.

use std::collections::BTreeSet;
use std::path::Path;

use thiserror::Error;

use crate::config::pack_cache::{TOML, tree_packs};
use crate::config::pack_registry::PACK_FILE;
use crate::config::pack_sources::FetchError;
use crate::config::pack_sources::hash::files;

/// The keys whose string value is a command line `mt` runs, as
/// `crate::allow::commands` lists them.
const COMMAND_KEYS: [&str; 2] = ["run", "editor"];

/// How an entry changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// It is new.
    Added,
    /// It holds another value.
    Changed,
    /// It is gone.
    Removed,
}

impl Change {
    /// `+`, `~` or `-`.
    #[must_use]
    pub const fn sign(self) -> char {
        match self {
            Self::Added => '+',
            Self::Changed => '~',
            Self::Removed => '-',
        }
    }
}

/// One changed key, rule or array entry of a pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryDiff {
    /// How it changed.
    pub change: Change,
    /// `rule <path>.<name>` for a rule, `<path>.<name>` or `<path>[<i>]` for
    /// another array entry, and `key <path>` for any other value.
    pub label: String,
    /// The command lines that are new in it, in full.
    pub commands: Vec<String>,
}

/// One pack of the tree that changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackDiff {
    /// The pack's name in the tree, without its source.
    pub pack: String,
    /// `Added` or `Removed` for a whole pack, `Changed` for one with
    /// [`PackDiff::entries`].
    pub change: Change,
    /// What changed inside a pack both trees hold.
    pub entries: Vec<EntryDiff>,
    /// For an added pack, every command line it carries.
    pub commands: Vec<String>,
}

/// What moving a source from one tree to another changes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeDiff {
    /// Each pack that changed, by name.
    pub packs: Vec<PackDiff>,
    /// Each other file that changed, by its `/`-separated path.
    pub files: Vec<(Change, String)>,
}

/// Why a tree could not be diffed.
#[derive(Debug, Error)]
pub enum DiffError {
    /// A file of the tree could not be listed or read.
    #[error("can't read the tree: {0}")]
    Read(FetchError),
    /// A pack file is not TOML.
    #[error("pack {pack} is not TOML: {error}")]
    Parse {
        /// The pack's name in the tree.
        pack: String,
        /// The parser's error.
        error: toml::de::Error,
    },
}

/// What changes from the tree at `old`, or an empty tree, to the one at `new`.
///
/// # Errors
/// Returns [`DiffError`] when a tree can't be read or a pack is not TOML.
pub fn diff_trees(old: Option<&Path>, new: &Path) -> Result<TreeDiff, DiffError> {
    let old = old.map(Tree::read).transpose()?.unwrap_or_default();
    let new = Tree::read(new)?;
    let names = union(
        old.packs.iter().map(|(name, _)| name),
        new.packs.iter().map(|(name, _)| name),
    );
    let packs = names
        .into_iter()
        .filter_map(|name| diff_pack(name, old.pack(name), new.pack(name)))
        .collect();
    let files = union(
        old.files.iter().map(|(path, _)| path),
        new.files.iter().map(|(path, _)| path),
    )
    .into_iter()
    .filter_map(|path| {
        let change = change_of(old.file(path), new.file(path))?;
        Some((change, path.to_owned()))
    })
    .collect();
    Ok(TreeDiff { packs, files })
}

/// What changed in the pack `name`, `None` when nothing did.
fn diff_pack(name: &str, old: Option<&toml::Table>, new: Option<&toml::Table>) -> Option<PackDiff> {
    let pack = |change, entries, commands| PackDiff {
        pack: name.to_owned(),
        change,
        entries,
        commands,
    };
    match (old, new) {
        (Some(old), Some(new)) => {
            let mut entries = Vec::new();
            diff_tables("", old, new, &mut entries);
            (!entries.is_empty()).then(|| pack(Change::Changed, entries, Vec::new()))
        }
        (None, Some(new)) => {
            let commands = commands_in(&toml::Value::Table(new.clone()));
            Some(pack(Change::Added, Vec::new(), commands))
        }
        (Some(_), None) => Some(pack(Change::Removed, Vec::new(), Vec::new())),
        (None, None) => None,
    }
}

/// A tree's packs, parsed, and its other files, read.
#[derive(Default)]
struct Tree {
    packs: Vec<(String, toml::Table)>,
    files: Vec<(String, Vec<u8>)>,
}

impl Tree {
    fn read(dir: &Path) -> Result<Self, DiffError> {
        let packs = tree_packs(dir);
        let mut tree = Self::default();
        for path in files(dir).map_err(DiffError::Read)? {
            let bytes =
                std::fs::read(dir.join(&path)).map_err(|error| DiffError::Read(error.into()))?;
            match packs.iter().find(|pack| is_pack_file(&path, pack)) {
                Some(pack) => {
                    let text = String::from_utf8_lossy(&bytes);
                    let table = toml::from_str(&text).map_err(|error| DiffError::Parse {
                        pack: pack.clone(),
                        error,
                    })?;
                    tree.packs.push((pack.clone(), table));
                }
                None => tree.files.push((path, bytes)),
            }
        }
        Ok(tree)
    }

    fn pack(&self, name: &str) -> Option<&toml::Table> {
        self.packs
            .iter()
            .find(|(pack, _)| pack == name)
            .map(|(_, table)| table)
    }

    fn file(&self, path: &str) -> Option<&[u8]> {
        self.files
            .iter()
            .find(|(file, _)| file == path)
            .map(|(_, bytes)| bytes.as_slice())
    }
}

/// Whether `path` is the TOML file of `pack`: `<pack>.toml` at the top, or
/// `<pack>/pack.toml`.
fn is_pack_file(path: &str, pack: &str) -> bool {
    path == format!("{pack}/{PACK_FILE}") || path == format!("{pack}{TOML}")
}

/// Every name of `a` and `b`, sorted, once.
fn union<'a>(
    a: impl Iterator<Item = &'a String>,
    b: impl Iterator<Item = &'a String>,
) -> BTreeSet<&'a str> {
    a.chain(b).map(String::as_str).collect()
}

/// The change from `old` to `new`, `None` when both hold the same.
fn change_of<T: PartialEq + ?Sized>(old: Option<&T>, new: Option<&T>) -> Option<Change> {
    match (old, new) {
        (None, Some(_)) => Some(Change::Added),
        (Some(_), None) => Some(Change::Removed),
        (Some(old), Some(new)) if old != new => Some(Change::Changed),
        _ => None,
    }
}

/// Push what changed from `old` to `new`, the tables at `path`.
fn diff_tables(path: &str, old: &toml::Table, new: &toml::Table, out: &mut Vec<EntryDiff>) {
    let keys: BTreeSet<&String> = old.keys().chain(new.keys()).collect();
    for key in keys {
        let at = if path.is_empty() {
            key.clone()
        } else {
            format!("{path}.{key}")
        };
        diff_values(&at, old.get(key), new.get(key), out);
    }
}

/// Push what changed from `old` to `new`, the values at `path`. A table and
/// an array of tables recurse, a missing one read as empty.
fn diff_values(
    path: &str,
    old: Option<&toml::Value>,
    new: Option<&toml::Value>,
    out: &mut Vec<EntryDiff>,
) {
    let empty = toml::Table::new();
    match (old, new) {
        (Some(toml::Value::Table(old)), Some(toml::Value::Table(new))) => {
            diff_tables(path, old, new, out);
        }
        (None, Some(toml::Value::Table(new))) => diff_tables(path, &empty, new, out),
        (Some(toml::Value::Table(old)), None) => diff_tables(path, old, &empty, out),
        _ => match (elements(old), elements(new)) {
            (Some(old), Some(new)) => diff_elements(path, &old, &new, out),
            (Some(old), None) if new.is_none() => diff_elements(path, &old, &[], out),
            (None, Some(new)) if old.is_none() => diff_elements(path, &[], &new, out),
            _ => push_leaf(path, old, new, out),
        },
    }
}

/// The tables of an array that holds nothing else, an empty one included,
/// each with its label.
fn elements(value: Option<&toml::Value>) -> Option<Vec<(String, &toml::Value)>> {
    let toml::Value::Array(items) = value? else {
        return None;
    };
    if !items.iter().all(toml::Value::is_table) {
        return None;
    }
    let named = items.iter().enumerate().map(|(index, item)| {
        let key = item.get("name").and_then(toml::Value::as_str);
        (
            key.map_or_else(|| format!("[{index}]"), |name| format!(".{name}")),
            item,
        )
    });
    Some(named.collect())
}

/// Push each element of `old` and `new`, arrays at `path`, that changed.
fn diff_elements(
    path: &str,
    old: &[(String, &toml::Value)],
    new: &[(String, &toml::Value)],
    out: &mut Vec<EntryDiff>,
) {
    let mut keys: Vec<&str> = new.iter().map(|(key, _)| key.as_str()).collect();
    keys.extend(
        old.iter()
            .map(|(key, _)| key.as_str())
            .filter(|key| find(new, key).is_none()),
    );
    let rules = path.rsplit('.').next() == Some("rules");
    for key in keys {
        let (before, after) = (find(old, key), find(new, key));
        let Some(change) = change_of(before, after) else {
            continue;
        };
        let label = if rules {
            format!("rule {path}{key}")
        } else {
            format!("{path}{key}")
        };
        out.push(EntryDiff {
            change,
            label,
            commands: new_commands(before, after),
        });
    }
}

/// The element of `items` labeled `key`.
fn find<'a>(items: &[(String, &'a toml::Value)], key: &str) -> Option<&'a toml::Value> {
    items
        .iter()
        .find(|(at, _)| at == key)
        .map(|(_, item)| *item)
}

/// Push the leaf at `path` when it changed.
fn push_leaf(
    path: &str,
    old: Option<&toml::Value>,
    new: Option<&toml::Value>,
    out: &mut Vec<EntryDiff>,
) {
    let Some(change) = change_of(old, new) else {
        return;
    };
    let key = path.rsplit('.').next().unwrap_or(path);
    let commands = match new.and_then(toml::Value::as_str) {
        Some(line) if COMMAND_KEYS.contains(&key) => vec![line.to_owned()],
        _ => Vec::new(),
    };
    out.push(EntryDiff {
        change,
        label: format!("key {path}"),
        commands,
    });
}

/// The command lines `new` holds that `old` doesn't.
fn new_commands(old: Option<&toml::Value>, new: Option<&toml::Value>) -> Vec<String> {
    let before = old.map(commands_in).unwrap_or_default();
    let after = new.map(commands_in).unwrap_or_default();
    after
        .into_iter()
        .filter(|line| !before.contains(line))
        .collect()
}

/// Every command line under `value`, in order.
fn commands_in(value: &toml::Value) -> Vec<String> {
    let mut lines = Vec::new();
    collect_commands(value, &mut lines);
    lines
}

fn collect_commands(value: &toml::Value, lines: &mut Vec<String>) {
    match value {
        toml::Value::Table(table) => {
            for (key, inner) in table {
                match inner.as_str() {
                    Some(line) if COMMAND_KEYS.contains(&key.as_str()) => {
                        lines.push(line.to_owned());
                    }
                    _ => collect_commands(inner, lines),
                }
            }
        }
        toml::Value::Array(items) => items.iter().for_each(|item| collect_commands(item, lines)),
        _ => {}
    }
}

#[cfg(test)]
#[path = "diff_tests.rs"]
mod tests;
