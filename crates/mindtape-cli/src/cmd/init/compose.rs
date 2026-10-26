//! Fold the `mt init` wizard's axis fragments into one config body.
//!
//! Each configuration axis the wizard walks answers with one TOML fragment
//! ([`crate::cmd::init::wizard`]); this module is the single place those fragments
//! become one `.mindtape/config.toml`. Keeping the merge in one function is
//! deliberate: presets reach the same [`compose`] when they carry an `[overlay]`
//! ([`crate::cmd::init::run`]'s `--preset` path), so composition never forks.
//!
//! Two rules make the fold safe. A fragment named `"overlay"` is the escape
//! hatch: it is deep-merged over the composed base last, so whatever the axes
//! cannot express (custom link kinds, named views, body-header rules) still wins.
//! And when two ordinary fragments both bind the same array-valued section — a
//! status legend, priority bands, difficulty anchors, roots — the fold hard-errors
//! naming both, because an array replaces wholesale and one would silently win.
//! The one array exempt from that rule is the top-level `extends`: each
//! fragment may adopt packs, and the fold joins their entries into one array.

use crate::diag::Result;

use crate::diag::{Diag, Hint, Note};

/// The reserved fragment name: its body is deep-merged over the composed base
/// last, and it alone is exempt from the array-conflict rule. The `--preset`
/// path in [`crate::cmd::init`] labels a preset's extracted overlay with it.
pub(crate) const OVERLAY_AXIS: &str = "overlay";

/// The top-level key a fragment adopts packs under. [`compose`] folds every
/// fragment's entries into one array.
const EXTENDS: &str = "extends";

/// Fold ordered fragments into one flattened config body.
///
/// Each fragment is a self-contained, readable TOML snippet that owns disjoint
/// sections: it uses only sub-table (`[tasks.priority]`) and array-of-table
/// (`[[tasks.status]]`) headers — never a bare `[tasks]` — so no two fragments
/// re-open the same table. The ordinary (non-overlay) fragments concatenate in
/// order, preserving each one's comments and layout, then re-parse as one TOML
/// document so a table two fragments both define surfaces here rather than at the
/// config loader.
///
/// A fragment's top-level `extends` leaves its place: the entries of every
/// fragment fold, in order and each once, into one `extends` array after the
/// comment block that opens the body. Left in place, a later fragment's
/// `extends` would land in the table an earlier one opened.
///
/// A fragment named `"overlay"` (see [`OVERLAY_AXIS`]) is not concatenated: its
/// body is deep-merged over the composed base after every other fragment, so it
/// beats them all, and the flattened result carries no literal `[overlay]`. When
/// an overlay is present the output is re-serialised from the merged document (so
/// comments do not survive the merge); with no overlay it is the annotated
/// concatenation unchanged.
///
/// # Errors
/// Returns an error when two ordinary fragments declare the same array-typed
/// section, when the concatenation is not one valid TOML document (two fragments
/// defined the same table), or when the overlay is not valid TOML.
pub fn compose(fragments: &[(&str, String)]) -> Result<String> {
    reject_array_conflicts(fragments)?;
    let (base, entries) = concat_base(fragments);
    let base = fold_extends(&base, &entries);
    let mut merged: toml::Table = toml::from_str(&base).map_err(|err| {
        Diag::failed("the axis fragments are not one TOML document")
            .cause(&err)
            .hint(Hint::new("two fragments may define the same table"))
    })?;
    let mut overlaid = false;
    for (axis, text) in fragments {
        if *axis != OVERLAY_AXIS {
            continue;
        }
        let overlay: toml::Table = toml::from_str(text)
            .map_err(|err| Diag::failed("the [overlay] fragment is not valid TOML").cause(&err))?;
        merge_overlay(&mut merged, overlay);
        overlaid = true;
    }
    if !overlaid {
        return Ok(base);
    }
    toml::to_string(&merged)
        .map_err(|err| Diag::failed("can't write the config with its overlay applied").cause(&err))
}

/// Concatenate the ordinary fragments in order, dropping empties and the overlay,
/// and lift out their `extends` entries, each once.
fn concat_base(fragments: &[(&str, String)]) -> (String, Vec<toml::Value>) {
    let mut out = String::new();
    let mut entries = Vec::new();
    for (axis, text) in fragments {
        if *axis == OVERLAY_AXIS {
            continue;
        }
        let text = lift_extends(text, &mut entries);
        let body = text.trim();
        if body.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(body);
        out.push('\n');
    }
    (out, entries)
}

/// Move a fragment's top-level `extends` entries into `entries`, skipping any
/// already there, and return the fragment without its `extends` line. A
/// fragment that does not parse, or adopts no pack, comes back unchanged.
fn lift_extends(text: &str, entries: &mut Vec<toml::Value>) -> String {
    let Ok(mut table) = toml::from_str::<toml::Table>(text) else {
        return text.to_owned();
    };
    let Some(toml::Value::Array(adopted)) = table.remove(EXTENDS) else {
        return text.to_owned();
    };
    for entry in adopted {
        if !entries.contains(&entry) {
            entries.push(entry);
        }
    }
    text.lines()
        .filter(|line| {
            !line
                .trim_start()
                .strip_prefix(EXTENDS)
                .is_some_and(|rest| rest.trim_start().starts_with('='))
        })
        .map(|line| format!("{line}\n"))
        .collect()
}

/// Write the folded `extends` array into `base`, after the comment block that
/// opens it, or first when no such block stands apart.
fn fold_extends(base: &str, entries: &[toml::Value]) -> String {
    if entries.is_empty() {
        return base.to_owned();
    }
    let listed: Vec<String> = entries.iter().map(ToString::to_string).collect();
    let (head, rest) = base.split_at(leading_comment_end(base));
    let mut out = String::from(head);
    if !head.is_empty() {
        out.push('\n');
    }
    out.push_str(&format!("{EXTENDS} = [{}]\n", listed.join(", ")));
    let rest = rest.trim_start_matches('\n');
    if !rest.is_empty() {
        out.push('\n');
        out.push_str(rest);
    }
    out
}

/// The byte offset just past the comment block that opens `text`, when a blank
/// line or the end closes it; else 0, since a comment right above a key
/// describes that key.
fn leading_comment_end(text: &str) -> usize {
    let mut end = 0;
    for line in text.split_inclusive('\n') {
        if line.trim_start().starts_with('#') {
            end += line.len();
            continue;
        }
        return if line.trim().is_empty() { end } else { 0 };
    }
    end
}

/// Refuse two ordinary fragments that each bind the same array-valued section.
///
/// Arrays replace wholesale under the overlay merge and append blindly under
/// plain concatenation, so neither expresses "two legends, one project" — the
/// only honest answer is to name the collision. The overlay is exempt: it exists
/// to override. The message names both fragments, the section, and the way out.
///
/// # Errors
/// Returns an error naming the two fragments and the array section they share.
fn reject_array_conflicts(fragments: &[(&str, String)]) -> Result<()> {
    let mut owner: std::collections::BTreeMap<String, &str> = std::collections::BTreeMap::new();
    for (axis, text) in fragments {
        if *axis == OVERLAY_AXIS {
            continue;
        }
        let Ok(table) = toml::from_str::<toml::Table>(text) else {
            continue;
        };
        let mut paths = Vec::new();
        collect_array_paths(&table, "", &mut paths);
        paths.retain(|path| path != EXTENDS);
        for path in paths {
            match owner.get(path.as_str()) {
                Some(prev) if *prev != *axis => {
                    return Err(
                        Diag::refused("two fragments declare the same array section")
                            .value(format!("[{path}]"))
                            .note(Note::new("declared by").name(format!("{prev} and {axis}")))
                            .hint(Hint::new("pick one, or move the difference into [overlay]")),
                    );
                }
                Some(_) => {}
                None => {
                    owner.insert(path, axis);
                }
            }
        }
    }
    Ok(())
}

/// Record every dotted path in `table` that binds an array, recursing through
/// sub-tables but never descending into an array's own elements (a `to =` list
/// inside one status entry is not a section of its own).
fn collect_array_paths(table: &toml::Table, prefix: &str, out: &mut Vec<String>) {
    for (key, value) in table {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        match value {
            toml::Value::Array(_) => out.push(path),
            toml::Value::Table(inner) => collect_array_paths(inner, &path, out),
            _ => {}
        }
    }
}

/// Overlay `top` onto `base`: two tables at the same key merge recursively;
/// anything else (a scalar, an array, or a table meeting a non-table) replaces
/// what the base held. This is the overlay-beats-base rule applied once, kept
/// CLI-side rather than reaching into mindtape's private merge.
fn merge_overlay(base: &mut toml::Table, top: toml::Table) {
    for (key, incoming) in top {
        match (base.get_mut(&key), incoming) {
            (Some(toml::Value::Table(nested)), toml::Value::Table(over)) => {
                merge_overlay(nested, over);
            }
            (_, incoming) => {
                base.insert(key, incoming);
            }
        }
    }
}

#[cfg(test)]
#[path = "compose_tests.rs"]
mod tests;
