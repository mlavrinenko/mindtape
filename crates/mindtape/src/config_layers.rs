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
//! the pure fold ([`merge_layers`]) so the merge order is testable without
//! touching a filesystem.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::{Config, ConfigError, EMBEDDED_DEFAULTS, config_path};
use crate::framework::merge_builtin_frameworks;
use crate::merge::deep_merge_tables;

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

/// Fold layers into one TOML document in the order given (later overrides
/// earlier), with [`deep_merge_tables`]. Pure — no IO.
///
/// # Errors
/// Returns [`ConfigError::Parse`] if any layer's text is not valid TOML.
pub fn merge_layers(layers: &[Layer]) -> Result<toml::Table, ConfigError> {
    let mut base = toml::Table::new();
    for layer in layers {
        let overlay: toml::Table = toml::from_str(&layer.text)?;
        deep_merge_tables(&mut base, overlay);
    }
    Ok(base)
}

/// [`merge_layers`], then splice in the built-in priority frameworks — the
/// merged document shared by [`build_config`] and [`merged_config_table`].
fn merge_doc(layers: &[Layer]) -> Result<toml::Table, ConfigError> {
    let mut doc = merge_layers(layers)?;
    merge_builtin_frameworks(&mut doc);
    Ok(doc)
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
fn build_config(layers: &[Layer], root: &Path) -> Result<Config, ConfigError> {
    let doc = merge_doc(layers)?;
    let mut config: Config = toml::Value::Table(doc).try_into()?;
    config.dict_cache = crate::config::compile_dicts(&config.dicts, root)
        .map_err(ConfigError::Invalid)?
        .into();
    crate::config_tasks::expand_dict_refs(&mut config.tasks, config.dict_cache.dicts())
        .map_err(ConfigError::Invalid)?;
    config.validate().map_err(ConfigError::Invalid)?;
    Ok(config)
}

/// Assemble the merged TOML document for `root` — every discovered layer
/// folded together with [`merge_layers`], plus the spliced-in built-in
/// priority frameworks — without deserializing into a typed [`Config`] or
/// running semantic validation. Backs `mt config show`: the emitted document
/// re-parses through [`crate::config::parse_config`] to an equal [`Config`].
///
/// # Errors
/// Returns [`ConfigError::Parse`] if any layer's text is not valid TOML.
pub fn merged_config_table(root: &Path) -> Result<toml::Table, ConfigError> {
    let layers = discover_layers(root, dirs::home_dir().as_deref());
    merge_doc(&layers)
}

/// Assemble the full layered config for `root`: discover every layer, merge
/// them, and build the typed [`Config`]. Returns the layer provenance
/// alongside the config, for `mt config show --origin`.
///
/// # Errors
/// Returns [`ConfigError`] if any layer fails to parse, the merged document
/// fails to deserialize, or semantic validation fails.
pub fn load_config_layered(root: &Path) -> Result<(Config, Vec<LayerSource>), ConfigError> {
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
) -> Result<(Config, Vec<LayerSource>), ConfigError> {
    let layers = discover_layers_with_xdg(root, home, xdg_config_dir);
    let sources = layers.iter().map(|layer| layer.source.clone()).collect();
    let config = build_config(&layers, root)?;
    Ok((config, sources))
}

#[cfg(test)]
#[path = "config_layers_tests.rs"]
mod tests;
