//! Named starter shapes for `mt init`.
//!
//! A preset is a `.mindtape/config.toml` body plus the `mt add` templates that
//! body needs. The two travel together on purpose: a preset that constrains
//! what a task must contain, without shipping a template that produces it,
//! scaffolds a project whose very first `mt add` fails validation.
//!
//! Presets are written annotated — every non-obvious choice carries the reason
//! for it — and [`strip_comments`] renders the quiet form. One source, two
//! renderings, so `--annotate` cannot drift from the default.
//!
//! Resolution order, highest first: a user preset under the XDG config dir, a
//! built-in, then an explicit path (an argument carrying a separator or a
//! `.toml` suffix). An unknown name is an error listing what exists — never a
//! fall-through to somewhere else, which is how `cargo-generate` turns a typo
//! into a git clone.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

/// The preset `mt init` writes when none is named.
pub const DEFAULT_PRESET: &str = "flow";

/// One built-in preset: its name, its config body, and its templates.
struct Builtin {
    name: &'static str,
    description: &'static str,
    config: &'static str,
    templates: &'static [(&'static str, &'static str)],
}

const BUILTINS: &[Builtin] = &[
    Builtin {
        name: "todone",
        description: "Open or finished, nothing else constrained",
        config: include_str!("presets/todone/config.toml"),
        templates: &[],
    },
    Builtin {
        name: "flow",
        description: "A lifecycle with enforced transitions, for solo or team work",
        config: include_str!("presets/flow/config.toml"),
        templates: &[],
    },
    Builtin {
        name: "personal",
        description: "Life admin: capture to an inbox, hide-until dates, weekly review",
        config: include_str!("presets/personal/config.toml"),
        templates: &[],
    },
    Builtin {
        name: "gated",
        description: "For AI agents: required spec sections, and a review state done cannot skip",
        config: include_str!("presets/gated/config.toml"),
        templates: &[("task.typ", include_str!("presets/gated/templates/task.typ"))],
    },
];

/// A resolved preset, ready to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preset {
    /// The name it resolved under, for messages.
    pub name: String,
    /// The annotated `.mindtape/config.toml` body.
    pub config: String,
    /// `mt add` templates to drop beside it, as `(file name, content)`.
    pub templates: Vec<(String, String)>,
}

impl Preset {
    /// The config body as written: annotated verbatim, or comment-stripped.
    #[must_use]
    pub fn render(&self, annotate: bool) -> String {
        if annotate {
            self.config.clone()
        } else {
            strip_comments(&self.config)
        }
    }
}

/// One line per built-in, for `--list-presets`.
#[must_use]
pub fn list_builtins() -> Vec<(&'static str, &'static str)> {
    BUILTINS
        .iter()
        .map(|builtin| (builtin.name, builtin.description))
        .collect()
}

/// Resolve `name` to a preset, searching the user preset directory first.
///
/// `user_dir` is the XDG preset directory (`dirs::config_dir()/mindtape/
/// presets`); pass `None` when it cannot be determined. Injected rather than
/// read here so tests need not touch the real environment.
///
/// # Errors
/// Returns an error if `name` matches no user preset, no built-in, and no
/// readable path.
pub fn resolve(name: &str, user_dir: Option<&Path>) -> Result<Preset> {
    if let Some(dir) = user_dir
        && let Some(preset) = user_preset(dir, name)?
    {
        return Ok(preset);
    }
    if let Some(builtin) = BUILTINS.iter().find(|builtin| builtin.name == name) {
        return Ok(Preset {
            name: builtin.name.to_owned(),
            config: builtin.config.to_owned(),
            templates: builtin
                .templates
                .iter()
                .map(|(file, body)| ((*file).to_owned(), (*body).to_owned()))
                .collect(),
        });
    }
    if looks_like_path(name) {
        return path_preset(Path::new(name));
    }
    bail!(
        "unknown preset {name:?}; built-in presets: {}",
        BUILTINS
            .iter()
            .map(|builtin| builtin.name)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Whether an argument should be treated as a filesystem path rather than a
/// name: it carries a path separator, or ends in `.toml`.
fn looks_like_path(name: &str) -> bool {
    name.contains(std::path::MAIN_SEPARATOR)
        || name.contains('/')
        || Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
}

/// The XDG user preset directory, if the config dir resolves.
#[must_use]
pub fn user_preset_dir() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("mindtape").join("presets"))
}

/// A user preset in `dir`, in either supported layout: `<name>.toml`, or
/// `<name>/config.toml` with an optional sibling `templates/` directory.
///
/// Returns `Ok(None)` when neither layout exists, so resolution falls through
/// to the built-ins.
fn user_preset(dir: &Path, name: &str) -> Result<Option<Preset>> {
    let flat = dir.join(format!("{name}.toml"));
    if flat.is_file() {
        let config = std::fs::read_to_string(&flat)
            .with_context(|| format!("read preset {}", flat.display()))?;
        return Ok(Some(Preset {
            name: name.to_owned(),
            config,
            templates: Vec::new(),
        }));
    }
    let nested = dir.join(name);
    if nested.join("config.toml").is_file() {
        return Ok(Some(read_preset_dir(name, &nested)?));
    }
    Ok(None)
}

/// An explicit path: a `.toml` file, or a directory laid out like a nested
/// user preset.
fn path_preset(path: &Path) -> Result<Preset> {
    let name = path.file_stem().map_or_else(
        || path.display().to_string(),
        |stem| stem.to_string_lossy().into_owned(),
    );
    if path.is_dir() {
        return read_preset_dir(&name, path);
    }
    let config =
        std::fs::read_to_string(path).with_context(|| format!("read preset {}", path.display()))?;
    Ok(Preset {
        name,
        config,
        templates: Vec::new(),
    })
}

/// Read the directory layout: `config.toml` plus every `templates/*.typ`.
fn read_preset_dir(name: &str, dir: &Path) -> Result<Preset> {
    let config_path = dir.join("config.toml");
    let config = std::fs::read_to_string(&config_path)
        .with_context(|| format!("read preset {}", config_path.display()))?;
    Ok(Preset {
        name: name.to_owned(),
        config,
        templates: read_templates(&dir.join("templates"))?,
    })
}

/// Every `*.typ` directly under `dir`, sorted by file name so a preset writes
/// the same bytes on every platform. A missing directory yields none.
fn read_templates(dir: &Path) -> Result<Vec<(String, String)>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("read templates {}", dir.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| format!("read templates {}", dir.display()))?
            .path();
        if path.extension().is_some_and(|ext| ext == "typ") {
            let file = path
                .file_name()
                .ok_or_else(|| anyhow!("template has no file name: {}", path.display()))?
                .to_string_lossy()
                .into_owned();
            let body = std::fs::read_to_string(&path)
                .with_context(|| format!("read template {}", path.display()))?;
            found.push((file, body));
        }
    }
    found.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(found)
}

/// Drop whole-line comments, then collapse the blank runs they leave behind.
///
/// Only lines whose first non-space character is `#` go, so a `#` inside a
/// string value survives. The result keeps at most one blank line in a row and
/// never opens with one, which is what makes the stripped rendering read as a
/// deliberate file rather than a gutted one.
#[must_use]
pub fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_blank = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.is_empty() {
            pending_blank = !out.is_empty();
            continue;
        }
        if pending_blank {
            out.push('\n');
            pending_blank = false;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
#[path = "preset_tests.rs"]
mod tests;
