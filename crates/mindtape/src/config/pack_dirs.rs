//! A directory pack's templates. After every layer and pack has merged,
//! [`append_template_dirs`] adds each directory pack's `templates/` to
//! `templates.dirs`, behind the project's own dirs, so a project template
//! with a pack template's name wins and `mt add --list` lists both.
//!
//! Template lookup takes the first dir holding a name, so the dirs go in
//! precedence order: the pack of the latest layer, and the last one in its
//! `extends`, first. That is the order a later rule replaces an earlier one
//! by name. A directory pack that two layers extend counts once, at its
//! latest place. The dirs are absolute, since a pack may sit outside the
//! project.

use std::path::{Component, Path, PathBuf};

use crate::config::ConfigError;
use crate::config::packs_toml::read_array;

/// The config path of the template dirs.
const TEMPLATE_DIRS: [&str; 2] = ["templates", "dirs"];

/// The folder of a directory pack that holds its templates.
const TEMPLATES_FOLDER: &str = "templates";

/// Append the `templates/` folder of each of `pack_dirs`, given in the order
/// the packs were folded, that exists, to the document's `templates.dirs`.
/// A document with no such folder to add is left untouched.
///
/// # Errors
/// Returns [`ConfigError::Invalid`] when `templates` isn't a table.
pub(super) fn append_template_dirs(
    doc: &mut toml::Table,
    pack_dirs: &[PathBuf],
) -> Result<(), ConfigError> {
    let mut added: Vec<String> = Vec::new();
    for dir in pack_dirs.iter().rev() {
        let templates = dir.join(TEMPLATES_FOLDER);
        if templates.is_dir() {
            let text = absolute(&templates).display().to_string();
            if !added.contains(&text) {
                added.push(text);
            }
        }
    }
    if added.is_empty() {
        return Ok(());
    }
    let mut dirs = read_array(doc, &TEMPLATE_DIRS);
    dirs.extend(added.into_iter().map(toml::Value::String));
    crate::config::packs_toml::write_array(doc, &TEMPLATE_DIRS, dirs)
}

/// `path` made absolute, with its `.` and `..` segments resolved by name, so
/// two spellings of one folder compare equal. The working directory is left
/// out when it can't be read.
fn absolute(path: &Path) -> PathBuf {
    let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let mut clean = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                if !clean.pop() {
                    clean.push(part);
                }
            }
            other => clean.push(other),
        }
    }
    clean
}
