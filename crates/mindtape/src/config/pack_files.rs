//! The files a pack names resolve from the pack's own directory, not the
//! project root: a rule's `check.typst` module and a dictionary's `path`.
//! A built-in pack has no directory, so its paths stay the project's.
//!
//! A dictionary keeps no origin once the packs merge, so [`rebase_dict_paths`]
//! turns each `path` into an absolute one while the pack is still its own
//! table, and `dicts.rs` reads an absolute path as it is. A rule does keep its
//! origin, so [`stamp_check_roots`] sets [`CheckSpec::root`] on each `check`
//! rule a pack declared, after the config is typed.

use std::path::Path;

use crate::config::pack_dirs::absolute;
use crate::config::pack_registry::{RuleOrigin, RuleOriginEntry};
use crate::rules::RuleSpec;

/// Make each `[dicts.<name>] path` of the pack table `table` absolute, from
/// the pack's directory `dir`.
pub(super) fn rebase_dict_paths(table: &mut toml::Table, dir: &Path) {
    let Some(dicts) = table.get_mut("dicts").and_then(toml::Value::as_table_mut) else {
        return;
    };
    for spec in dicts.iter_mut().filter_map(|(_, spec)| spec.as_table_mut()) {
        if let Some(path) = spec.get_mut("path")
            && let Some(text) = path.as_str()
        {
            *path = toml::Value::String(absolute(&dir.join(text)).display().to_string());
        }
    }
}

/// Set the root of each `check` rule of `rules` that a pack declared, to
/// that pack's directory. `origins` lists where every rule came from, in the
/// order of the merged arrays, `[[tasks.rules]]` first.
pub(crate) fn stamp_check_roots(rules: &mut [RuleSpec], origins: &[RuleOriginEntry]) {
    for (spec, entry) in rules.iter_mut().zip(origins) {
        if let (Some(check), RuleOrigin::Pack(pack)) = (&mut spec.check, &entry.origin) {
            check.root = pack.root().map(Path::to_path_buf);
        }
    }
}
