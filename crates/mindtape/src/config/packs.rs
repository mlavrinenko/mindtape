//! `extends`/`disable`: folding each layer's packs in just below it, and
//! resolving the five arrays that merge by name, `[[tasks.rules]]`,
//! `[[tasks.body.headers.rules]]`, `[[tasks.body.content.rules]]`,
//! `[[commits.rules]]` and `[[on.flip]]`, across every layer and pack.
//!
//! A pack is a config-syntax fragment. `<source>:<pack>` resolves to one
//! compiled into the binary ([`EMBEDDED_PACKS`], under `mindtape`); a path
//! (anything with a path separator) resolves to a TOML file, or to a
//! directory holding `pack.toml`, relative to the layer that names it. Before
//! any layer folds, [`crate::config::pack_sources::for_layers`] reads the
//! project's `[pack-sources]` and checks them against `.mindtape/packs.lock`,
//! and only the project layer may name a pack in one. After the merge,
//! [`append_template_dirs`] adds each directory pack's `templates/` to
//! `templates.dirs`. A pack may declare any config key but the few
//! [`crate::config::pack_resolve`] refuses, `extends` among them, so
//! resolution is one level deep.
//!
//! Per layer, in order: [`fold_layer_packs`] resolves the layer's `extends`
//! entries, takes each pack's rule arrays out, and folds the rest into one
//! [`PackLayer`] sub-layer ([`crate::config::pack_fold`] holds the merge and
//! its conflict check, which a key the layer sets itself settles). The
//! sub-layer deep-merges into the document, then the layer's own keys on
//! top, so a layer's table keeps the keys its packs set and it doesn't.
//!
//! That loop in [`fold_layer_packs`] is the seam later steps hook into. A
//! step that rewrites a whole pack, rule expressions included, sits between
//! [`resolve_pack`] and [`take_rule_arrays`], and takes out any key of its
//! own before [`PackLayer::add`] merges the rest, as [`apply_params`] does
//! with `[params]`. A layer's own table gets [`refuse_param_calls`] first,
//! since no parameter reaches it. A step that needs each pack's own table
//! reads [`PackLayer`]'s per-pack tables before [`PackLayer::into_table`]
//! drops them, as [`Sources::record_keys`] does for the keys each pack set.
//!
//! Per rule array, entries fold in source order — every pack a layer extends
//! (in `extends` order), then the layer's own declared entries — and a later
//! entry sharing an earlier one's `name` replaces it IN PLACE rather than
//! appending a second copy. That replacement only ever looks at entries from
//! an EARLIER source: two same-named entries within one source's own array
//! are both kept untouched, so a same-file mistake still reaches
//! [`crate::body::headers::validate_header_rules`] /
//! [`crate::body::content::validate_content_rules`] as a duplicate-name error
//! instead of silently collapsing. `disable` then drops every entry (in any of
//! the five arrays) carrying a listed name, erroring on a name matching
//! none.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::config::ConfigError;
use crate::config::layers::{Layer, LayerSource};
use crate::config::locate::Sources;
use crate::config::merge::deep_merge_tables;
use crate::config::pack_dirs::append_template_dirs;
use crate::config::pack_files::rebase_dict_paths;
use crate::config::pack_fold::PackLayer;
use crate::config::pack_registry::{EMBEDDED_PACKS, RuleOrigin, RuleOriginEntry};
use crate::config::pack_resolve::{PackScope, resolve_pack};
use crate::config::packs_toml::{
    ExtendsEntry, header_lines, read_array, read_extends, read_string_list, remove_path,
    write_array,
};
use crate::config::params::{apply_params, refuse_param_calls};

/// The array TOML paths that append across layers and packs,
/// same-name-replaces, rather than replacing wholesale like every other
/// array.
const RULE_ARRAY_PATHS: [&[&str]; RULE_ARRAYS] = [
    &["tasks", "rules"],
    &["tasks", "body", "headers", "rules"],
    &["tasks", "body", "content", "rules"],
    &["commits", "rules"],
    &["on", "flip"],
];

/// How many rule arrays merge by name.
const RULE_ARRAYS: usize = 5;

/// The dotted report prefix for each of [`RULE_ARRAY_PATHS`], in the same
/// order.
const RULE_ARRAY_PREFIXES: [&str; RULE_ARRAYS] = [
    "tasks.rules",
    "tasks.body.headers.rules",
    "tasks.body.content.rules",
    "commits.rules",
    "on.flip",
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
    /// The text of every layer and pack file, in merge precedence, so an
    /// error can name the file it comes from.
    pub sources: Sources,
}

/// Fold `layers` into one document, resolving each layer's own `extends`
/// (against [`EMBEDDED_PACKS`]) and applying its `disable` after every layer
/// has merged in.
///
/// # Errors
/// Returns [`ConfigError::Invalid`] for a malformed `extends`/`disable`
/// entry, a misplaced or stale pack source, an unresolvable pack, a pack declaring a key it may not, or a
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
    let sources = crate::config::pack_sources::for_layers(layers, root)?;
    let mut pack_dirs: Vec<PathBuf> = Vec::new();
    let mut files = Sources::default();

    for layer in layers {
        let mut table: toml::Table =
            toml::from_str(&layer.text).map_err(|error| layer_parse_error(&layer.source, error))?;
        let extends = read_extends(&table)?;
        disable_names.extend(read_string_list(&table, "disable")?);
        refuse_param_calls(&mut table)?;

        let base_dir = layer_base_dir(&layer.source, root);
        let scope = PackScope {
            registry,
            sources: &sources,
            project: matches!(layer.source, LayerSource::Project(_)),
        };
        let packs = fold_layer_packs(&extends, &base_dir, &scope, &table, &mut accumulators)?;
        pack_dirs.extend(packs.dirs().map(Path::to_path_buf));
        files.push_packs(packs.texts(), root);
        files.push_layer(layer, root);
        for (pack, params) in packs.params() {
            files.record_params(&layer.source, pack, params);
        }
        for (pack, pack_table) in packs.tables() {
            files.record_keys(&RuleOrigin::Pack(pack.clone()), pack_table);
        }
        deep_merge_tables(&mut doc, packs.into_table());

        let origin = RuleOrigin::Layer(layer.source.clone());
        take_rule_arrays(&mut table, &layer.text, &origin, &mut accumulators);
        files.record_keys(&origin, &table);
        deep_merge_tables(&mut doc, table);
    }

    apply_disable(&mut accumulators, &disable_names)?;
    append_template_dirs(&mut doc, &pack_dirs)?;

    let mut rule_origins = Vec::new();
    for ((accumulator, path), prefix) in accumulators
        .into_iter()
        .zip(RULE_ARRAY_PATHS)
        .zip(RULE_ARRAY_PREFIXES)
    {
        let (values, origins) = accumulator.into_parts();
        files.add_rule_slots(
            path,
            origins.iter().map(|(_, origin, line)| (origin, *line)),
        );
        write_array(&mut doc, path, values)?;
        rule_origins.extend(labeled_origins(prefix, origins));
    }

    Ok(PackMerge {
        table: doc,
        rule_origins,
        sources: files,
    })
}

/// Resolve one layer's `extends` entries in order: each pack's `param()`
/// calls take the entry's values, its rule arrays join `accumulators`, and
/// the rest of it folds into the [`PackLayer`] that merges just below
/// `layer`, the layer's own table.
///
/// # Errors
/// Returns [`ConfigError`] for an unresolvable pack, one declaring a key a
/// pack may not, a parameter error, one pack listed twice, or two packs
/// setting one value differently that `layer` doesn't set.
fn fold_layer_packs(
    extends: &[ExtendsEntry],
    base_dir: &Path,
    scope: &PackScope<'_>,
    layer: &toml::Table,
    accumulators: &mut [RuleAccumulator; RULE_ARRAYS],
) -> Result<PackLayer, ConfigError> {
    let mut packs = PackLayer::default();
    for entry in extends {
        let mut pack = resolve_pack(&entry.pack, base_dir, scope)?;
        if let Some(dir) = pack.origin.root() {
            rebase_dict_paths(&mut pack.table, dir);
        }
        let params = apply_params(&mut pack.table, &entry.with, &pack.origin.label())?;
        packs.keep_params(pack.origin.clone(), params);
        let origin = RuleOrigin::Pack(pack.origin.clone());
        take_rule_arrays(&mut pack.table, &pack.text, &origin, accumulators);
        packs.keep_text(pack.origin.clone(), pack.text);
        packs.add(pack.origin, pack.table, layer)?;
    }
    Ok(packs)
}

/// Move one source's rule arrays out of `table` into `accumulators`, so the
/// deep merge of the rest never replaces one source's array with another's.
/// `text` is the source's TOML, for each rule's header line.
fn take_rule_arrays(
    table: &mut toml::Table,
    text: &str,
    origin: &RuleOrigin,
    accumulators: &mut [RuleAccumulator; RULE_ARRAYS],
) {
    for (accumulator, path) in accumulators.iter_mut().zip(RULE_ARRAY_PATHS) {
        let lines = header_lines(text, path);
        accumulator.extend_batch(read_array(table, path), origin, &lines);
        remove_path(table, path);
    }
}

/// One accumulated rule: its value, the source it came from and its header line.
type Placed = (toml::Value, RuleOrigin, Option<usize>);

/// One rule array's accumulated entries: `name`-indexed so a later source's
/// entry sharing an earlier one's name replaces it in place.
#[derive(Default)]
struct RuleAccumulator {
    entries: Vec<Placed>,
    index_by_name: HashMap<String, usize>,
}

impl RuleAccumulator {
    /// Fold one source's own array in, in order. A name this accumulator
    /// already placed (from an EARLIER source) is replaced in place; a name
    /// repeated within THIS batch is left alone — see the module doc for why.
    /// `lines` holds each entry's header line, and is used only when it counts
    /// the batch exactly.
    fn extend_batch(&mut self, batch: Vec<toml::Value>, origin: &RuleOrigin, lines: &[usize]) {
        let mut seen_this_batch: HashSet<String> = HashSet::new();
        let exact = lines.len() == batch.len();
        for (position, value) in batch.into_iter().enumerate() {
            let line = lines.get(position).copied().filter(|_| exact);
            let name = value.get("name").and_then(toml::Value::as_str);
            let Some(name) = name.filter(|name| seen_this_batch.insert((*name).to_owned())) else {
                self.entries.push((value, origin.clone(), line));
                continue;
            };
            if let Some(index) = self.index_by_name.get(name).copied() {
                if let Some(slot) = self.entries.get_mut(index) {
                    *slot = (value, origin.clone(), line);
                }
            } else {
                self.index_by_name
                    .insert(name.to_owned(), self.entries.len());
                self.entries.push((value, origin.clone(), line));
            }
        }
    }

    /// Drop every entry named `name`, across the whole array. Returns whether
    /// any were removed.
    fn remove_named(&mut self, name: &str) -> bool {
        let before = self.entries.len();
        self.entries
            .retain(|(value, _, _)| value.get("name").and_then(toml::Value::as_str) != Some(name));
        let removed = self.entries.len() != before;
        if removed {
            self.index_by_name.clear();
            for (index, (value, _, _)) in self.entries.iter().enumerate() {
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
    fn into_parts(self) -> (Vec<toml::Value>, Vec<NamedOrigin>) {
        let mut values = Vec::with_capacity(self.entries.len());
        let mut named_origins = Vec::with_capacity(self.entries.len());
        for (value, origin, line) in self.entries {
            let name = value
                .get("name")
                .and_then(toml::Value::as_str)
                .map(str::to_owned);
            named_origins.push((name, origin, line));
            values.push(value);
        }
        (values, named_origins)
    }
}

/// A rule's `name` (if it has one), its origin and its header line.
type NamedOrigin = (Option<String>, RuleOrigin, Option<usize>);

/// One `RuleOriginEntry` per entry in `named_origins`, labeled `prefix.<name>`
/// when named, else `prefix[<index>]` — matching
/// [`crate::rules::RuleSpec::label`]'s own name-or-position fallback.
fn labeled_origins(prefix: &str, named_origins: Vec<NamedOrigin>) -> Vec<RuleOriginEntry> {
    named_origins
        .into_iter()
        .enumerate()
        .map(|(index, (name, origin, line))| RuleOriginEntry {
            label: match name {
                Some(name) => format!("{prefix}.{name}"),
                None => format!("{prefix}[{index}]"),
            },
            origin,
            line,
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
            return Err(ConfigError::invalid(format!(
                "disable: no rule named: {}",
                super::quote(name)
            )));
        }
    }
    Ok(())
}

/// A layer's TOML error, naming the file unless it is the project's own (the
/// CLI already names that one) or the embedded floor.
fn layer_parse_error(source: &LayerSource, error: toml::de::Error) -> ConfigError {
    match source {
        LayerSource::Xdg(path) | LayerSource::Dir(path) => ConfigError::ParseIn {
            origin: path.display().to_string(),
            error,
        },
        LayerSource::Project(_) | LayerSource::Embedded => ConfigError::Parse(error),
    }
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
#[path = "packs_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "packs_tests_body.rs"]
mod tests_body;

#[cfg(test)]
#[path = "packs_tests_extends.rs"]
mod tests_extends;

#[cfg(test)]
#[path = "packs_tests_merge.rs"]
mod tests_merge;

#[cfg(test)]
#[path = "packs_tests_dirs.rs"]
mod tests_dirs;

#[cfg(test)]
#[path = "packs_tests_files.rs"]
mod tests_files;

#[cfg(test)]
#[path = "packs_tests_flip.rs"]
mod tests_flip;

#[cfg(test)]
#[path = "packs_tests_params.rs"]
mod tests_params;

#[cfg(test)]
#[path = "packs_tests_budget.rs"]
mod tests_budget;

#[cfg(test)]
#[path = "packs_tests_commits.rs"]
mod tests_commits;

#[cfg(test)]
#[path = "packs_tests_origins.rs"]
mod tests_origins;
