//! `add` support: value sources, template resolution, describe/list output.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result, bail};
use mindtape::{Config, RawValue, SlotSpec};

/// Built-in default task template, used when no `task.typ` is found under the
/// configured template dirs.
pub const BUILTIN_TASK: &str = include_str!("templates/task.typ");

/// How `describe` renders a template's slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescribeFormat {
    /// One human-readable line per slot.
    Text,
    /// A JSON array of slot objects.
    Json,
}

impl DescribeFormat {
    /// Parse the `--describe[=FORMAT]` value.
    ///
    /// # Errors
    /// Returns an error for an unknown format.
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => bail!("unknown describe format {other:?}; expected text or json"),
        }
    }
}

/// Read a template by stem from the configured dirs, falling back to the
/// built-in `task` template.
///
/// # Errors
/// Returns an error when a named template is absent and is not the built-in.
pub fn resolve_template(config: &Config, root: &Path, name: &str) -> Result<String> {
    for dir in &config.templates.dirs {
        let path = root.join(dir).join(format!("{name}.typ"));
        if path.is_file() {
            return std::fs::read_to_string(&path)
                .with_context(|| format!("read {}", path.display()));
        }
    }
    if name == "task" {
        return Ok(BUILTIN_TASK.to_owned());
    }
    bail!(
        "template {name:?} not found under {:?}",
        config.templates.dirs
    )
}

/// Available template stems: every `*.typ` under the configured dirs plus the
/// built-in `task`, deduplicated and sorted.
#[must_use]
pub fn list_templates(config: &Config, root: &Path) -> Vec<String> {
    let mut names = std::collections::BTreeSet::new();
    names.insert("task".to_owned());
    for dir in &config.templates.dirs {
        let Ok(entries) = std::fs::read_dir(root.join(dir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("typ")
                && let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
            {
                names.insert(stem.to_owned());
            }
        }
    }
    names.into_iter().collect()
}

/// Merge JSON `--values` sources (paths; `-` = stdin), then apply `--set k=v`
/// string overrides, into a typed fill map.
///
/// # Errors
/// Returns an error when a source is unreadable, not a JSON object, holds an
/// unsupported value shape, or a `--set` entry lacks `=`.
pub fn collect_values(values: &[String], sets: &[String]) -> Result<BTreeMap<String, RawValue>> {
    let mut merged = serde_json::Map::new();
    for source in values {
        let text = read_source(source)?;
        let parsed: serde_json::Value =
            serde_json::from_str(&text).with_context(|| format!("parse JSON from {source}"))?;
        let object = parsed
            .as_object()
            .with_context(|| format!("values from {source} must be a JSON object"))?;
        for (key, value) in object {
            merged.insert(key.clone(), value.clone());
        }
    }
    let mut out = BTreeMap::new();
    for (key, value) in &merged {
        if let Some(raw) = json_to_raw(value).with_context(|| format!("value for {key:?}"))? {
            out.insert(key.clone(), raw);
        }
    }
    for set in sets {
        let (key, value) = set
            .split_once('=')
            .with_context(|| format!("--set must be key=value, got {set:?}"))?;
        out.insert(key.to_owned(), RawValue::Str(value.to_owned()));
    }
    Ok(out)
}

fn read_source(source: &str) -> Result<String> {
    if source == "-" {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .context("read values from stdin")?;
        return Ok(text);
    }
    std::fs::read_to_string(source).with_context(|| format!("read values from {source}"))
}

fn json_to_raw(value: &serde_json::Value) -> Result<Option<RawValue>> {
    let raw = match value {
        serde_json::Value::Null => return Ok(None),
        serde_json::Value::String(text) => RawValue::Str(text.clone()),
        serde_json::Value::Bool(flag) => RawValue::Bool(*flag),
        serde_json::Value::Number(num) => num.as_i64().map_or_else(
            || RawValue::Float(num.as_f64().unwrap_or_default()),
            RawValue::Int,
        ),
        serde_json::Value::Array(items) => {
            let mut list = Vec::with_capacity(items.len());
            for item in items {
                let text = item
                    .as_str()
                    .context("array values must be strings")?
                    .to_owned();
                list.push(text);
            }
            RawValue::List(list)
        }
        serde_json::Value::Object(_) => bail!("nested object values are unsupported"),
    };
    Ok(Some(raw))
}

/// Render a template's slots for `--describe`.
///
/// # Errors
/// Returns an error only when JSON serialization fails.
pub fn describe(specs: &[SlotSpec], format: DescribeFormat) -> Result<String> {
    match format {
        DescribeFormat::Text => Ok(describe_text(specs)),
        DescribeFormat::Json => describe_json(specs),
    }
}

fn describe_text(specs: &[SlotSpec]) -> String {
    let mut lines = Vec::new();
    for spec in specs {
        let need = if spec.required && spec.default.is_none() {
            "required"
        } else {
            "optional"
        };
        let mut line = format!("{}  {}  {need}", spec.name, spec.kind.as_str());
        if let Some(default) = &spec.default {
            line.push_str(&format!("  = {default}"));
        }
        if let Some(prompt) = &spec.prompt {
            line.push_str(&format!("  # {prompt}"));
        }
        lines.push(line);
    }
    lines.join("\n")
}

fn describe_json(specs: &[SlotSpec]) -> Result<String> {
    let rows: Vec<serde_json::Value> = specs
        .iter()
        .map(|spec| {
            serde_json::json!({
                "name": spec.name,
                "kind": spec.kind.as_str(),
                "required": spec.required && spec.default.is_none(),
                "default": spec.default,
                "prompt": spec.prompt,
                "choices": spec.choices,
            })
        })
        .collect();
    serde_json::to_string_pretty(&rows).context("serialize slot description")
}

/// Slugify `text` into a filename stem: lowercase ASCII alphanumerics, other
/// runs collapsed to single `-`, trimmed.
#[must_use]
pub fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.extend(ch.to_lowercase());
        } else {
            pending_dash = true;
        }
    }
    out
}

#[cfg(test)]
#[path = "add_values_tests.rs"]
mod tests;
