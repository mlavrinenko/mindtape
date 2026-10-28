//! The files a pack names resolve from the pack's own directory, not the
//! project root: a rule's `check.typst` module and a dictionary's `path`.
//! A built-in pack has no directory, so its paths stay the project's. A
//! cached pack's paths must stay inside its source's tree, which came from
//! the network.
//!
//! A dictionary keeps no origin once the packs merge, so [`rebase_dict_paths`]
//! turns each `path` into an absolute one while the pack is still its own
//! table, and `dicts.rs` reads an absolute path as it is. A rule does keep its
//! origin, so [`stamp_check_roots`] sets [`CheckSpec::root`] on each `check`
//! rule a pack declared, after the config is typed.

use std::path::Path;

use crate::config::pack_dirs::{absolute, stays_inside};
use crate::config::pack_registry::{PackOrigin, RuleOrigin, RuleOriginEntry};
use crate::config::{ConfigError, quote};
use crate::rules::RuleSpec;

/// Make each `[dicts.<name>] path` of the pack table `table` absolute, from
/// the pack's directory `dir`.
///
/// # Errors
/// Returns [`ConfigError::Invalid`] when `origin` has a
/// [`PackOrigin::bound`] and a path leaves it.
pub(super) fn rebase_dict_paths(
    table: &mut toml::Table,
    dir: &Path,
    origin: &PackOrigin,
) -> Result<(), ConfigError> {
    let Some(dicts) = table.get_mut("dicts").and_then(toml::Value::as_table_mut) else {
        return Ok(());
    };
    for (name, spec) in dicts.iter_mut() {
        let Some(path) = spec.as_table_mut().and_then(|spec| spec.get_mut("path")) else {
            continue;
        };
        let Some(text) = path.as_str() else {
            continue;
        };
        if let Some(bound) = origin.bound()
            && !stays_inside(bound, &dir.join(text))
        {
            return Err(ConfigError::invalid(format!(
                "extends: pack {} names a file outside its source: dicts.{name}.path: {}",
                quote(&origin.label()),
                quote(text)
            )));
        }
        *path = toml::Value::String(absolute(&dir.join(text)).to_string_lossy().into_owned());
    }
    Ok(())
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
