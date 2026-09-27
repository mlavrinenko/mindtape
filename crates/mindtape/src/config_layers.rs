//! Multi-file config layer discovery and merge: the assembly around
//! [`crate::config::parse_config`]'s embedded-floor-plus-one-string core.
//!
//! Precedence, lowest to highest:
//! 1. The compiled-in embedded floor (`config.rs`'s `EMBEDDED_DEFAULTS`).
//! 2. XDG global defaults: `<config home>/mindtape/defaults.toml`, with the
//!    config home resolved by [`crate::app_dirs::config_home`].
//! 3. Every `.mindtape/defaults.toml` found walking up from the project root,
//!    farthest ancestor first, so the nearest one wins.
//! 4. The project's own `<root>/.mindtape/config.toml`.
//!
//! IO (locating and reading files, [`discover_layers`]) is kept separate from
//! the fold: [`crate::config_packs::merge_layers_with_packs`] takes the
//! discovered [`Layer`]s and does the rest, so the merge order is testable
//! without touching a filesystem (see that module's own tests). Every field
//! wholesale-replaces a later layer's over an earlier one's, except
//! `[[tasks.rules]]` and its two body-rule siblings, which append across
//! layers (and across a layer's own `extends` packs) instead — that
//! exception is entirely that module's, not this one's.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::{Config, ConfigError, EMBEDDED_DEFAULTS, config_path};
use crate::config_packs::{RuleOriginEntry, merge_layers_with_packs};
use crate::framework::merge_builtin_frameworks;

/// One config text layer plus where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    /// Where this layer's text was read from (or [`LayerSource::Embedded`]).
    pub source: LayerSource,
    /// Raw TOML text, not yet parsed.
    pub text: String,
}

/// Provenance for one merged-in config layer. Backs `mt config show --origin`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerSource {
    /// The compiled-in default floor.
    Embedded,
    /// `<config home>/mindtape/defaults.toml`.
    Xdg(PathBuf),
    /// One ancestor's `.mindtape/defaults.toml`, found walking up from root.
    Dir(PathBuf),
    /// The project's own `.mindtape/config.toml`.
    Project(PathBuf),
}

/// Locate every config layer that applies to `root`, in precedence order
/// (lowest first). Reads the real global config dir
/// ([`crate::app_dirs::config_home`]); see [`load_config_layered`] for the entry point
/// that also supplies the real home directory.
///
/// `home` bounds the ancestor walk-up (never ascended past); pass
/// `dirs::home_dir()` in production, an injected path in tests.
#[must_use]
pub fn discover_layers(root: &Path, home: Option<&Path>) -> Vec<Layer> {
    discover_layers_with_xdg(root, home, crate::app_dirs::config_home().as_deref())
}

/// [`discover_layers`] with the XDG global config dir injected, so tests can
/// point it at a tempdir instead of depending on the real environment.
fn discover_layers_with_xdg(
    root: &Path,
    home: Option<&Path>,
    xdg_config_dir: Option<&Path>,
) -> Vec<Layer> {
    let mut layers = vec![Layer {
        source: LayerSource::Embedded,
        text: EMBEDDED_DEFAULTS.to_owned(),
    }];
    layers.extend(xdg_layer(xdg_config_dir));
    layers.extend(dir_layers(root, home));
    layers.extend(project_layer(root));
    layers
}

/// The XDG global defaults layer, if the config home resolved and
/// `mindtape/defaults.toml` exists under it.
fn xdg_layer(xdg_config_dir: Option<&Path>) -> Option<Layer> {
    let path = xdg_config_dir?.join("mindtape").join("defaults.toml");
    let text = std::fs::read_to_string(&path).ok()?;
    Some(Layer {
        source: LayerSource::Xdg(path),
        text,
    })
}

/// The project's own `.mindtape/config.toml`, if it exists.
fn project_layer(root: &Path) -> Option<Layer> {
    let path = config_path(root);
    let text = std::fs::read_to_string(&path).ok()?;
    Some(Layer {
        source: LayerSource::Project(path),
        text,
    })
}

/// Every ancestor `.mindtape/defaults.toml` from `root` up to (and including)
/// `home`, or the filesystem root when `root` is not under `home`. Returned
/// farthest-ancestor first, so the nearest overlays last (wins). Paths are
/// canonicalised before comparison and de-duplication, so a symlinked home (or
/// a symlink loop in the ancestry) does not double-count a layer.
fn dir_layers(root: &Path, home: Option<&Path>) -> Vec<Layer> {
    let canonical_home = home.and_then(|h| h.canonicalize().ok());
    let mut dir = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    loop {
        if !seen.insert(dir.clone()) {
            break;
        }
        let candidate = dir.join(".mindtape").join("defaults.toml");
        if let Ok(text) = std::fs::read_to_string(&candidate) {
            found.push(Layer {
                source: LayerSource::Dir(candidate),
                text,
            });
        }
        if canonical_home.as_deref() == Some(dir.as_path()) {
            break;
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => break,
        }
    }
    found.reverse();
    found
}

/// [`crate::config_packs::merge_layers_with_packs`] (deep-merges every layer,
/// folding in each one's own `extends` packs and `disable` names as it goes),
/// then splice in the built-in priority frameworks — the merged document,
/// plus every rule's provenance, shared by [`build_config`] and
/// [`merged_config_table`].
///
/// # Errors
/// Returns [`ConfigError`] if any layer or pack fails to parse, an `extends`
/// pack can't be resolved, or a `disable` name matches no rule.
fn merge_doc(
    layers: &[Layer],
    root: &Path,
) -> Result<(toml::Table, Vec<RuleOriginEntry>), ConfigError> {
    let mut merged = merge_layers_with_packs(layers, root)?;
    merge_builtin_frameworks(&mut merged.table);
    Ok((merged.table, merged.rule_origins))
}

/// [`merge_doc`], then deserialize and run semantic validation — the shared
/// tail with [`crate::config::parse_config`], plus the checks that need the
/// project root.
///
/// `[dicts.<name>]` is compiled here and kept on the config: a `path` entry
/// resolves against `root`, so this is the first point that can read one, and
/// the whole point of compiling at load is that a broken dictionary is one
/// config error instead of one per task. The result lands in
/// [`Config::compiled_dicts`] rather than being dropped, because the filter
/// language's `in_dict()` runs wherever a filter runs — a view, a limit, a
/// `required` rule, a `[[tasks.rules]]` assertion, an `[[on.flip]]` gate — and
/// none of those call sites has a project root to re-read the section with.
///
/// It compiles BEFORE [`Config::validate`], not after. That pass dry-runs
/// every saved filter expression against a probe task, so an expression
/// reaching a project dictionary — `in_dict("verbs", word(title, 1))`, the
/// canonical rule — is checked against a config whose only dictionaries are
/// the built-ins if the cache is still empty, and fails at load naming a dict
/// the project plainly declares. Nothing is lost by the order:
/// [`crate::config::compile_dicts`] runs `validate_dicts` itself, so a broken
/// `[dicts]` section is still refused, by the first pass that can see it.
///
/// [`crate::config_tasks::expand_dict_refs`] runs right after, for the same
/// reason: a `@<dict>` reference in `[tasks.tags] allowed` or
/// `[tasks.fields.<name>] values` needs the compiled dictionaries to resolve
/// against, and needs to have resolved before `validate` (which does not
/// itself read either list, but every command downstream of a successful load
/// does) ever sees a bare `@name` instead of the set it names.
fn build_config(
    layers: &[Layer],
    root: &Path,
) -> Result<(Config, Vec<RuleOriginEntry>), ConfigError> {
    let (doc, rule_origins) = merge_doc(layers, root)?;
    let mut config: Config = toml::Value::Table(doc).try_into()?;
    config.dict_cache = crate::config::compile_dicts(&config.dicts, root)
        .map_err(ConfigError::Invalid)?
        .into();
    crate::config_tasks::expand_dict_refs(&mut config.tasks, config.dict_cache.dicts())
        .map_err(ConfigError::Invalid)?;
    // Stamped before `validate` reads it (transitively, through a
    // `[[tasks.rules]] check` entry's `CompiledCheck`) and before the
    // root-aware `validate_load` below, which takes `root` directly rather
    // than reading it back off `config` — the two would otherwise have to
    // agree on an ordering that only exists to satisfy this line.
    config.rules_root.set(root);
    config.git.set_root(root);
    config.validate().map_err(ConfigError::Invalid)?;
    // Root-aware, so it cannot run inside `Config::validate` itself (which
    // stays callable rootless, e.g. from `mt init`'s wizard and from the many
    // unit tests that build a `Config` from a bare TOML string): a
    // `[[tasks.rules]] check` entry's Typst module and function are only
    // reachable once a project root is in hand, exactly the reason
    // `compile_dicts` above already runs here rather than in `parse_config`.
    // Runs over `config.tasks.rules` after packs and every layer have merged
    // in, so a `check` rule a pack or an ancestor's `.mindtape/defaults.toml`
    // contributed is validated here too, against the same `root` as any
    // other — `rules_root` carries no provenance, only the one project root
    // `build_config` was ever called with.
    crate::config_rules_typst::validate_load(&config, root).map_err(ConfigError::Invalid)?;
    Ok((config, rule_origins))
}

/// Assemble the merged TOML document for `root` — every discovered layer (and
/// each one's `extends` packs) folded together, plus the spliced-in built-in
/// priority frameworks — without deserializing into a typed [`Config`] or
/// running semantic validation. Backs `mt config show`: the emitted document
/// re-parses through [`crate::config::parse_config`] to an equal [`Config`].
///
/// # Errors
/// As [`merge_doc`].
pub fn merged_config_table(root: &Path) -> Result<toml::Table, ConfigError> {
    let layers = discover_layers(root, dirs::home_dir().as_deref());
    merge_doc(&layers, root).map(|(table, _rule_origins)| table)
}

/// Assemble the full layered config for `root`: discover every layer, merge
/// them (resolving each one's `extends` packs and applying `disable`), and
/// build the typed [`Config`]. Returns the layer provenance and every rule's
/// own, for `mt config show --origin`.
///
/// # Errors
/// Returns [`ConfigError`] if any layer or pack fails to parse, an `extends`
/// pack can't be resolved, a `disable` name matches no rule, the merged
/// document fails to deserialize, or semantic validation fails.
pub fn load_config_layered(
    root: &Path,
) -> Result<(Config, Vec<LayerSource>, Vec<RuleOriginEntry>), ConfigError> {
    load_config_layered_from(
        root,
        dirs::home_dir().as_deref(),
        crate::app_dirs::config_home().as_deref(),
    )
}

/// [`load_config_layered`] with both environment-derived directories injected:
/// `home` bounds the ancestor walk, `xdg_config_dir` locates the global
/// `mindtape/defaults.toml` layer. Pass `None` for either to drop it.
///
/// The entry point for a caller that must not read the running machine —
/// embedders wiring their own config discovery, and any test asserting on a
/// full merge, which through [`load_config_layered`] would instead assert on
/// whether the developer keeps a global `defaults.toml`.
///
/// # Errors
/// As [`load_config_layered`].
pub fn load_config_layered_from(
    root: &Path,
    home: Option<&Path>,
    xdg_config_dir: Option<&Path>,
) -> Result<(Config, Vec<LayerSource>, Vec<RuleOriginEntry>), ConfigError> {
    let layers = discover_layers_with_xdg(root, home, xdg_config_dir);
    let sources = layers.iter().map(|layer| layer.source.clone()).collect();
    let (config, rule_origins) = build_config(&layers, root)?;
    Ok((config, sources, rule_origins))
}

#[cfg(test)]
#[path = "config_layers_tests.rs"]
mod tests;
