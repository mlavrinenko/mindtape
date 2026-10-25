//! Nested-path reads and writes over a raw `toml::Table`, shared by
//! [`crate::config::packs`]'s pre-typed-`Config` merge: reading and stripping
//! a layer's own rule arrays before the generic fold, and splicing the
//! resolved ones back in afterward. Split out of `config/packs.rs` to keep
//! that module's own pack-resolution and merge logic under the file-size cap.

use crate::config::ConfigError;

/// Read a top-level `Vec<String>` key (`extends`/`disable`), defaulting to
/// empty when absent.
///
/// # Errors
/// Returns [`ConfigError::Invalid`] if the key exists but isn't an array of
/// strings.
pub(crate) fn read_string_list(table: &toml::Table, key: &str) -> Result<Vec<String>, ConfigError> {
    match table.get(key) {
        None => Ok(Vec::new()),
        Some(toml::Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| ConfigError::invalid(format!("{key}: entries must be strings")))
            })
            .collect(),
        Some(_) => Err(ConfigError::invalid(format!(
            "{key}: must be an array of strings"
        ))),
    }
}

/// The array at a nested TOML path (e.g. `["tasks", "body", "headers",
/// "rules"]`), or empty when any segment along the way is missing or not a
/// table/array.
pub(crate) fn read_array(table: &toml::Table, path: &[&str]) -> Vec<toml::Value> {
    let Some((&last, ancestors)) = path.split_last() else {
        return Vec::new();
    };
    let mut current: &toml::Table = table;
    for key in ancestors {
        match current.get(*key).and_then(toml::Value::as_table) {
            Some(next) => current = next,
            None => return Vec::new(),
        }
    }
    current
        .get(last)
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// The value at a nested TOML path, or `None` when any segment along the way
/// is missing or not a table.
pub(crate) fn get_path<'a>(table: &'a toml::Table, path: &[&str]) -> Option<&'a toml::Value> {
    let (&last, ancestors) = path.split_last()?;
    let mut current = table;
    for key in ancestors {
        current = current.get(*key)?.as_table()?;
    }
    current.get(last)
}

/// The one-based line of each `[[<path>]]` table header in `text`, in file
/// order — the position of every entry [`read_array`] reads from a file
/// declaring its rules as tables. A file that writes the array inline has no
/// header to point at, so the caller checks the count against the array.
pub(crate) fn header_lines(text: &str, path: &[&str]) -> Vec<usize> {
    let header = format!("[[{}]]", path.join("."));
    text.lines()
        .enumerate()
        .filter(|(_, line)| {
            line.split('#')
                .next()
                .is_some_and(|code| code.trim() == header)
        })
        .map(|(index, _)| index + 1)
        .collect()
}

/// Remove whatever sits at a nested TOML path, leaving any intermediate
/// tables in place (only the leaf key is dropped).
pub(crate) fn remove_path(table: &mut toml::Table, path: &[&str]) {
    let Some((&last, ancestors)) = path.split_last() else {
        return;
    };
    let mut current = table;
    for key in ancestors {
        match current.get_mut(*key).and_then(toml::Value::as_table_mut) {
            Some(next) => current = next,
            None => return,
        }
    }
    current.remove(last);
}

/// Write `values` as the array at a nested TOML path, creating any missing
/// intermediate table.
///
/// # Errors
/// Returns [`ConfigError::Invalid`] if an existing value along the path is
/// not a table.
pub(crate) fn write_array(
    table: &mut toml::Table,
    path: &[&str],
    values: Vec<toml::Value>,
) -> Result<(), ConfigError> {
    let Some((&last, ancestors)) = path.split_last() else {
        return Ok(());
    };
    let mut current = table;
    for key in ancestors {
        let entry = current
            .entry((*key).to_owned())
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        current = entry
            .as_table_mut()
            .ok_or_else(|| ConfigError::invalid(format!("{key}: expected a table")))?;
    }
    current.insert(last.to_owned(), toml::Value::Array(values));
    Ok(())
}

#[cfg(test)]
#[path = "packs_toml_tests.rs"]
mod tests;
