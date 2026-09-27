//! `extends`/`disable`: resolving rule packs into a layer's own
//! `[[tasks.rules]]`, `[[tasks.body.headers.rules]]`,
//! `[[tasks.body.content.rules]]` and `[[commits.rules]]` before
//! [`crate::config_layers`] deep-merges everything else.
//!
//! A pack is a config-syntax fragment carrying only the four rule arrays and
//! `[dicts]` — a bare name resolves to one compiled into the binary
//! ([`EMBEDDED_PACKS`]); a path (anything with a path separator, even one
//! starting with the reserved `mindtape-` prefix) resolves to a TOML file,
//! relative to the layer that names it. A pack may not itself `extends`, so
//! resolution is one level deep.
//!
//! Per rule array, entries fold in source order — every pack a layer extends
//! (in `extends` order), then the layer's own declared entries — and a later
//! entry sharing an earlier one's `name` replaces it IN PLACE rather than
//! appending a second copy. That replacement only ever looks at entries from
//! an EARLIER source: two same-named entries within one source's own array
//! are both kept untouched, so a same-file mistake still reaches
//! [`crate::body_headers::validate_header_rules`] /
//! [`crate::body_content::validate_content_rules`] as a duplicate-name error
//! instead of silently collapsing. `disable` then drops every rule (in any of
//! the four arrays) carrying a listed name, erroring on a name matching
//! none.
//!
//! `[dicts]` needs no such array-splicing: each pack's `[dicts]` table is
//! deep-merged in ahead of the extending layer's own document, so the normal
//! per-key table merge already gives "later (the layer's own) wins".
//! Everything else a pack might contain is rejected at load — see
//! [`validate_pack_shape`].

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::config::ConfigError;
use crate::config_layers::{Layer, LayerSource};
use crate::config_packs_toml::{read_array, read_string_list, remove_path, write_array};
use crate::merge::deep_merge_tables;

/// Rule packs compiled into the binary, `(name, TOML text)`. A new entry is a
/// one-line addition here plus a `crates/mindtape/src/packs/<name>.toml`
/// file, named under the reserved `mindtape-` prefix.
pub(crate) const EMBEDDED_PACKS: &[(&str, &str)] =
    &[("mindtape-tasks", include_str!("packs/mindtape-tasks.toml"))];

/// Where one resolved pack came from, for `mt config show --origin`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackOrigin {
    /// A bare name resolved against [`EMBEDDED_PACKS`].
    Embedded(String),
    /// A path pack, resolved to an absolute (or as-given, if the base
    /// couldn't be made absolute) path.
    Path(PathBuf),
}

/// Where one entry in a merged rule array came from: a layer file directly,
/// or a pack that layer's `extends` pulled in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleOrigin {
    /// Declared directly in this config layer.
    Layer(LayerSource),
    /// Pulled in through `extends`.
    Pack(PackOrigin),
}

/// One rule's provenance in a merged array: its report-style label
/// (`tasks.rules.<name>` or `tasks.rules[<index>]`, matching
/// [`crate::config_rules::RuleSpec::label`]'s own convention) and where it
/// came from. Backs `mt config show --origin`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleOriginEntry {
    /// e.g. `tasks.rules.imperative-title` or `tasks.body.headers.rules[2]`.
    pub label: String,
    /// The pack or layer this entry's current content came from.
    pub origin: RuleOrigin,
}

/// The rule-array TOML paths that append across layers and packs,
/// same-name-replaces, rather than replacing wholesale like every other
/// array.
const RULE_ARRAY_PATHS: [&[&str]; RULE_ARRAYS] = [
    &["tasks", "rules"],
    &["tasks", "body", "headers", "rules"],
    &["tasks", "body", "content", "rules"],
    &["commits", "rules"],
];

/// How many rule arrays merge by name.
const RULE_ARRAYS: usize = 4;

/// The dotted report prefix for each of [`RULE_ARRAY_PATHS`], in the same
/// order.
const RULE_ARRAY_PREFIXES: [&str; RULE_ARRAYS] = [
    "tasks.rules",
    "tasks.body.headers.rules",
    "tasks.body.content.rules",
    "commits.rules",
];

/// The merged document plus every rule's provenance, [`merge_layers_with_packs`]'s
/// result.
#[derive(Debug)]
pub(crate) struct PackMerge {
    /// The merged TOML document — every layer and pack folded together, the
    /// rule arrays already resolved.
    pub table: toml::Table,
    /// One entry per final rule, across every rule array, in array order.
    pub rule_origins: Vec<RuleOriginEntry>,
}

/// Fold `layers` into one document, resolving each layer's own `extends`
/// (against [`EMBEDDED_PACKS`]) and applying its `disable` after every layer
/// has merged in.
///
/// # Errors
/// Returns [`ConfigError::Invalid`] for a malformed `extends`/`disable`
/// entry, an unresolvable pack, a pack declaring a key it may not, or a
/// `disable` name matching no rule.
pub(crate) fn merge_layers_with_packs(
    layers: &[Layer],
    root: &Path,
) -> Result<PackMerge, ConfigError> {
    merge_layers_with_packs_using(layers, root, EMBEDDED_PACKS)
}

/// [`merge_layers_with_packs`] with the embedded-pack registry injected, so a
/// test can resolve a bare name against a fixture pack without touching the
/// real (possibly empty) [`EMBEDDED_PACKS`].
fn merge_layers_with_packs_using(
    layers: &[Layer],
    root: &Path,
    registry: &[(&str, &str)],
) -> Result<PackMerge, ConfigError> {
    let mut doc = toml::Table::new();
    let mut accumulators: [RuleAccumulator; RULE_ARRAYS] = Default::default();
    let mut disable_names: BTreeSet<String> = BTreeSet::new();

    for layer in layers {
        let mut table: toml::Table = toml::from_str(&layer.text)?;
        let extends = read_string_list(&table, "extends")?;
        disable_names.extend(read_string_list(&table, "disable")?);

        let base_dir = layer_base_dir(&layer.source, root);
        for entry in &extends {
            let pack = resolve_pack(entry, &base_dir, registry)?;
            let origin = RuleOrigin::Pack(pack.origin);
            for (accumulator, path) in accumulators.iter_mut().zip(RULE_ARRAY_PATHS) {
                accumulator.extend_batch(read_array(&pack.table, path), &origin);
            }
            if let Some(dicts) = pack.table.get("dicts") {
                let mut wrapper = toml::Table::new();
                wrapper.insert("dicts".to_owned(), dicts.clone());
                deep_merge_tables(&mut doc, wrapper);
            }
        }

        let origin = RuleOrigin::Layer(layer.source.clone());
        for (accumulator, path) in accumulators.iter_mut().zip(RULE_ARRAY_PATHS) {
            accumulator.extend_batch(read_array(&table, path), &origin);
        }
        // The rule-array paths merge specially (above); strip them so the
        // generic fold below never wholesale-replaces one with another
        // layer's, only to be overwritten again once every layer is in.
        for path in RULE_ARRAY_PATHS {
            remove_path(&mut table, path);
        }
        deep_merge_tables(&mut doc, table);
    }

    apply_disable(&mut accumulators, &disable_names)?;

    let mut rule_origins = Vec::new();
    for ((accumulator, path), prefix) in accumulators
        .into_iter()
        .zip(RULE_ARRAY_PATHS)
        .zip(RULE_ARRAY_PREFIXES)
    {
        let (values, origins) = accumulator.into_parts();
        write_array(&mut doc, path, values)?;
        rule_origins.extend(labeled_origins(prefix, origins));
    }

    Ok(PackMerge {
        table: doc,
        rule_origins,
    })
}

/// One rule array's accumulated entries: `name`-indexed so a later source's
/// entry sharing an earlier one's name replaces it in place.
#[derive(Default)]
struct RuleAccumulator {
    entries: Vec<(toml::Value, RuleOrigin)>,
    index_by_name: HashMap<String, usize>,
}

impl RuleAccumulator {
    /// Fold one source's own array in, in order. A name this accumulator
    /// already placed (from an EARLIER source) is replaced in place; a name
    /// repeated within THIS batch is left alone — see the module doc for why.
    fn extend_batch(&mut self, batch: Vec<toml::Value>, origin: &RuleOrigin) {
        let mut seen_this_batch: HashSet<String> = HashSet::new();
        for value in batch {
            let name = value.get("name").and_then(toml::Value::as_str);
            let Some(name) = name.filter(|name| seen_this_batch.insert((*name).to_owned())) else {
                self.entries.push((value, origin.clone()));
                continue;
            };
            if let Some(index) = self.index_by_name.get(name).copied() {
                if let Some(slot) = self.entries.get_mut(index) {
                    *slot = (value, origin.clone());
                }
            } else {
                self.index_by_name
                    .insert(name.to_owned(), self.entries.len());
                self.entries.push((value, origin.clone()));
            }
        }
    }

    /// Drop every entry named `name`, across the whole array. Returns whether
    /// any were removed.
    fn remove_named(&mut self, name: &str) -> bool {
        let before = self.entries.len();
        self.entries
            .retain(|(value, _)| value.get("name").and_then(toml::Value::as_str) != Some(name));
        let removed = self.entries.len() != before;
        if removed {
            self.index_by_name.clear();
            for (index, (value, _)) in self.entries.iter().enumerate() {
                if let Some(name) = value.get("name").and_then(toml::Value::as_str) {
                    self.index_by_name.insert(name.to_owned(), index);
                }
            }
        }
        removed
    }

    /// Split into the plain values (for splicing back into the merged
    /// document) and each one's `name` (if it has one) paired with its
    /// origin (for [`labeled_origins`]).
    fn into_parts(self) -> (Vec<toml::Value>, Vec<(Option<String>, RuleOrigin)>) {
        let mut values = Vec::with_capacity(self.entries.len());
        let mut named_origins = Vec::with_capacity(self.entries.len());
        for (value, origin) in self.entries {
            let name = value
                .get("name")
                .and_then(toml::Value::as_str)
                .map(str::to_owned);
            named_origins.push((name, origin));
            values.push(value);
        }
        (values, named_origins)
    }
}

/// One `RuleOriginEntry` per entry in `named_origins`, labeled `prefix.<name>`
/// when named, else `prefix[<index>]` — matching
/// [`crate::config_rules::RuleSpec::label`]'s own name-or-position fallback.
fn labeled_origins(
    prefix: &str,
    named_origins: Vec<(Option<String>, RuleOrigin)>,
) -> Vec<RuleOriginEntry> {
    named_origins
        .into_iter()
        .enumerate()
        .map(|(index, (name, origin))| RuleOriginEntry {
            label: match name {
                Some(name) => format!("{prefix}.{name}"),
                None => format!("{prefix}[{index}]"),
            },
            origin,
        })
        .collect()
}

/// Drop every rule named in `disable` from any of the accumulators.
/// Deduplicated first, so the same name declared in two layers' `disable`
/// (both legitimately met by the first removal) doesn't fail the second look.
fn apply_disable(
    accumulators: &mut [RuleAccumulator; RULE_ARRAYS],
    disable: &BTreeSet<String>,
) -> Result<(), ConfigError> {
    for name in disable {
        let removed = accumulators
            .iter_mut()
            .fold(false, |found, acc| acc.remove_named(name) || found);
        if !removed {
            return Err(ConfigError::Invalid(format!(
                "disable: no rule named `{name}`"
            )));
        }
    }
    Ok(())
}

/// One resolved pack: its provenance and validated table.
struct ResolvedPack {
    origin: PackOrigin,
    table: toml::Table,
}

/// A path pack has a path separator somewhere in it — a bare embedded-pack
/// name never does, even one carrying the reserved `mindtape-` prefix
/// (`"mindtape-tasks.toml"` with no separator is still a bare name, and an
/// unknown one).
fn is_path_entry(entry: &str) -> bool {
    entry.chars().any(std::path::is_separator)
}

/// Resolve one `extends` entry to its pack table, validating its shape.
fn resolve_pack(
    entry: &str,
    base_dir: &Path,
    registry: &[(&str, &str)],
) -> Result<ResolvedPack, ConfigError> {
    if is_path_entry(entry) {
        let path = base_dir.join(entry);
        let text = std::fs::read_to_string(&path).map_err(|_| {
            ConfigError::Invalid(format!("extends: pack not found: {}", path.display()))
        })?;
        let table: toml::Table = toml::from_str(&text)?;
        validate_pack_shape(&table, &path.display().to_string())?;
        Ok(ResolvedPack {
            origin: PackOrigin::Path(path),
            table,
        })
    } else {
        let text = registry
            .iter()
            .find(|(name, _)| *name == entry)
            .map(|(_, text)| *text)
            .ok_or_else(|| unknown_pack_error(entry, registry))?;
        let table: toml::Table = toml::from_str(text)?;
        validate_pack_shape(&table, entry)?;
        Ok(ResolvedPack {
            origin: PackOrigin::Embedded(entry.to_owned()),
            table,
        })
    }
}

fn unknown_pack_error(entry: &str, registry: &[(&str, &str)]) -> ConfigError {
    let available = if registry.is_empty() {
        "(none)".to_owned()
    } else {
        registry
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
            .join(", ")
    };
    ConfigError::Invalid(format!(
        "extends: unknown pack `{entry}`; available packs: {available}"
    ))
}

/// A pack may declare only `[[tasks.rules]]`, `[[tasks.body.headers.rules]]`,
/// `[[tasks.body.content.rules]]`, `[[commits.rules]]` and `[dicts]` — nested
/// exactly that deep, nothing else, including `extends` itself, so resolution
/// stays one level deep.
fn validate_pack_shape(table: &toml::Table, pack_label: &str) -> Result<(), ConfigError> {
    reject_unknown_keys(table, &["tasks", "commits", "dicts"], pack_label)?;
    if let Some(commits) = table.get("commits").and_then(toml::Value::as_table) {
        reject_unknown_keys(commits, &["rules"], pack_label)?;
    }
    let Some(tasks) = table.get("tasks").and_then(toml::Value::as_table) else {
        return Ok(());
    };
    reject_unknown_keys(tasks, &["rules", "body"], pack_label)?;
    let Some(body) = tasks.get("body").and_then(toml::Value::as_table) else {
        return Ok(());
    };
    reject_unknown_keys(body, &["headers", "content"], pack_label)?;
    if let Some(headers) = body.get("headers").and_then(toml::Value::as_table) {
        reject_unknown_keys(headers, &["rules"], pack_label)?;
    }
    if let Some(content) = body.get("content").and_then(toml::Value::as_table) {
        reject_unknown_keys(content, &["rules"], pack_label)?;
    }
    Ok(())
}

fn reject_unknown_keys(
    table: &toml::Table,
    allowed: &[&str],
    pack_label: &str,
) -> Result<(), ConfigError> {
    for key in table.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(ConfigError::Invalid(format!(
                "pack `{pack_label}` may not declare `{key}`"
            )));
        }
    }
    Ok(())
}

/// The directory a layer's OWN relative paths resolve against: every layer
/// file lives two segments under it (`<dir>/.mindtape/config.toml`,
/// `<dir>/.mindtape/defaults.toml`, or XDG's `<config-home>/mindtape/
/// defaults.toml`), so its grandparent is that directory — `root` itself for
/// the project layer, matching the task's worked example. Falls back to
/// `root` for the embedded floor (which carries no path, and never declares
/// `extends`) or a layer path that turns out to be too shallow.
fn layer_base_dir(source: &LayerSource, root: &Path) -> PathBuf {
    let path = match source {
        LayerSource::Xdg(path) | LayerSource::Dir(path) | LayerSource::Project(path) => Some(path),
        LayerSource::Embedded => None,
    };
    path.and_then(|p| p.parent())
        .and_then(Path::parent)
        .map_or_else(|| root.to_path_buf(), Path::to_path_buf)
}

#[cfg(test)]
#[path = "config_packs_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "config_packs_tests_body.rs"]
mod tests_body;

#[cfg(test)]
#[path = "config_packs_tests_extends.rs"]
mod tests_extends;
