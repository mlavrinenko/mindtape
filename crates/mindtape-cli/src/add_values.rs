//! `add` support: value sources, template resolution, describe/list output.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result, bail};
use mindtape::{Config, RawValue, SlotSpec};

use crate::add_tokens::apply_token;
use crate::yaml_values;

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
    if let Some(path) = template_path(config, root, name) {
        return std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()));
    }
    if name == "task" {
        return Ok(BUILTIN_TASK.to_owned());
    }
    bail!(
        "template {name:?} not found under {:?}",
        config.templates.dirs
    )
}

/// The file template `name` resolves to: the first configured dir holding
/// `<name>.typ`. `None` when no dir does, which for `task` means the built-in.
#[must_use]
pub fn template_path(config: &Config, root: &Path, name: &str) -> Option<std::path::PathBuf> {
    (config.templates.dirs.iter())
        .map(|dir| root.join(dir).join(format!("{name}.typ")))
        .find(|path| path.is_file())
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

/// Merge `--values` sources (paths; `-` = stdin; JSON or YAML, see
/// [`parse_source`]), then apply the `key OP value` tokens — `--set` entries
/// first, then the bare arguments after `--` — into a typed fill map. The
/// token grammar itself (`=`, `+=`, `@`) lives in [`crate::add_tokens`].
///
/// Sources are applied left to right, key by key: a later source's value for
/// a key replaces an earlier one, and an explicit `null`/`~` in a later
/// source unsets a key an earlier source set (rather than being silently
/// skipped) — the same "last source wins" rule extended to an explicit
/// unset, uniformly across JSON and YAML sources.
///
/// # Errors
/// Returns an error when a source is unreadable, not a JSON object or YAML
/// mapping, holds an unsupported value shape, or a token is not a valid
/// assignment (see [`apply_token`]).
pub fn collect_values(
    values: &[String],
    sets: &[String],
    tokens: &[String],
) -> Result<BTreeMap<String, RawValue>> {
    let mut out = BTreeMap::new();
    for source in values {
        let text = read_source(source)?;
        let patch = parse_source(source, &text)?;
        for (key, value) in patch {
            match value {
                Some(raw) => {
                    out.insert(key, raw);
                }
                None => {
                    out.remove(&key);
                }
            }
        }
    }
    for token in sets.iter().chain(tokens) {
        apply_token(&mut out, token)?;
    }
    Ok(out)
}

/// Reject any `values` key that names no slot in `specs` — a `--set`/`-f`
/// typo would otherwise silently vanish (the template default fills in
/// instead) with nothing in the output to notice. Mirrors the existing
/// list-slot-misuse hard error: a caller mistake fails loudly, naming the
/// fix, rather than filing a task with the wrong value.
///
/// # Errors
/// Returns an error listing the unknown key(s) and the template's known slot
/// names when any `values` key matches no discovered slot.
pub fn reject_unknown_keys(values: &BTreeMap<String, RawValue>, specs: &[SlotSpec]) -> Result<()> {
    let known: Vec<&str> = specs.iter().map(|spec| spec.name.as_str()).collect();
    let unknown: Vec<&str> = values
        .keys()
        .map(String::as_str)
        .filter(|key| !known.contains(key))
        .collect();
    if unknown.is_empty() {
        return Ok(());
    }
    let named: Vec<String> = unknown
        .iter()
        .map(|key| format!("{key}{}", mindtape::suggest::suggestion(key, &known)))
        .collect();
    bail!(
        "unknown --set/-f key(s): {}; known slots: {}",
        named.join(", "),
        known.join(", ")
    );
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

/// Parse one `-f/--values` source into a key -> (value or explicit unset)
/// patch. Dispatch is per source: `.yaml`/`.yml` and `.json` extensions pick
/// their parser outright; stdin (`-`) and any other extension sniff the
/// content instead, since neither carries a filename extension to trust.
fn parse_source(source: &str, text: &str) -> Result<BTreeMap<String, Option<RawValue>>> {
    if source == "-" {
        return parse_sniffed_source(source, text);
    }
    match Path::new(source).extension().and_then(|ext| ext.to_str()) {
        Some("yaml" | "yml") => parse_yaml_source(source, text),
        Some("json") => parse_json_source(source, text),
        _ => parse_sniffed_source(source, text),
    }
}

/// Sniff JSON vs. YAML from the content: a leading `{`/`[` is tried as JSON
/// first (retrying YAML only if that fails — a YAML flow-mapping edge case);
/// anything else goes straight to YAML. When both attempts fail, the JSON
/// parse error is reported: the leading brace/bracket was a strong signal of
/// intent, and re-reading the same malformed bytes as YAML tends to produce
/// the more confusing of the two errors.
fn parse_sniffed_source(source: &str, text: &str) -> Result<BTreeMap<String, Option<RawValue>>> {
    if !looks_like_json(text) {
        return parse_yaml_source(source, text);
    }
    match parse_json_source(source, text) {
        Ok(map) => Ok(map),
        Err(json_err) => parse_yaml_source(source, text).map_err(|_| json_err),
    }
}

fn looks_like_json(text: &str) -> bool {
    matches!(text.trim_start().as_bytes().first(), Some(b'{' | b'['))
}

fn parse_json_source(source: &str, text: &str) -> Result<BTreeMap<String, Option<RawValue>>> {
    let parsed: serde_json::Value =
        serde_json::from_str(text).with_context(|| format!("parse JSON from {source}"))?;
    let object = parsed
        .as_object()
        .with_context(|| format!("values from {source} must be a JSON object"))?;
    let mut out = BTreeMap::new();
    for (key, value) in object {
        let raw = json_to_raw(value).with_context(|| format!("value for {key:?}"))?;
        out.insert(key.clone(), raw);
    }
    Ok(out)
}

fn parse_yaml_source(source: &str, text: &str) -> Result<BTreeMap<String, Option<RawValue>>> {
    yaml_values::parse_mapping(
        text,
        || format!("parse YAML from {source}"),
        || format!("values from {source} must be a mapping"),
    )
}

/// Extract a task title from raw stdin text: the first non-blank line, trimmed.
/// A title is single-line, so trailing lines are ignored. Returns `None` when
/// the input holds no non-blank line.
#[must_use]
pub fn stdin_title(raw: &str) -> Option<String> {
    raw.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

/// The environment variable that fills slot `name`: `MINDTAPE_ADD_` plus the
/// upper-cased name with `-` turned into `_` (so `depends-on` reads
/// `MINDTAPE_ADD_DEPENDS_ON`).
#[must_use]
pub fn env_var_name(slot: &str) -> String {
    format!("MINDTAPE_ADD_{}", slot.to_uppercase().replace('-', "_"))
}

/// Fill each slot left unset by `--set`/`-f` from its environment variable via
/// `lookup`: the declared `env` name when the slot names one, else the implicit
/// `MINDTAPE_ADD_*` variable. Values are strings (coerced to the slot kind by
/// `fill`, like `--set`); blank values are skipped. Reading per discovered slot
/// keeps the mapping unambiguous and ignores unknown env keys.
pub fn layer_env(
    values: &mut BTreeMap<String, RawValue>,
    specs: &[SlotSpec],
    lookup: impl Fn(&str) -> Option<String>,
) {
    for spec in specs {
        if values.contains_key(&spec.name) {
            continue;
        }
        let key = match &spec.env {
            Some(var) => var.clone(),
            None => env_var_name(&spec.name),
        };
        if let Some(value) = lookup(&key)
            && !value.trim().is_empty()
        {
            values.insert(spec.name.clone(), RawValue::Str(value));
        }
    }
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
        if let Some(env) = &spec.env {
            line.push_str(&format!("  env: {env}"));
        }
        if let Some(default) = &spec.default {
            line.push_str(&format!("  = {default}"));
        }
        if let Some(prompt) = &spec.prompt {
            line.push_str(&format!("  # {prompt}"));
        }
        if let Some(example) = example_of(spec) {
            line.push_str(&format!(
                "\n    e.g. {}",
                example.replace('\n', "\n         ")
            ));
        }
        lines.push(line);
    }
    lines.join("\n")
}

/// A slot's example as a caller would type it: a string's or raw block's
/// text, else the literal's source.
fn example_of(spec: &SlotSpec) -> Option<&str> {
    spec.example_string.as_deref().or(spec.example.as_deref())
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
                "example": example_of(spec),
                "prompt": spec.prompt,
                "env": spec.env,
                "choices": spec.choices,
            })
        })
        .collect();
    serde_json::to_string_pretty(&rows).context("serialize slot description")
}

/// Slugify `text` into a filename stem: `mindtape::slug_from_title`'s rule,
/// shared with `mt import` so both mint identical stems from an identical
/// title.
#[must_use]
pub fn slug(text: &str) -> String {
    mindtape::slug_from_title(text)
}

#[cfg(test)]
#[path = "add_values_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "add_values_tests_env.rs"]
mod tests_env;

#[cfg(test)]
#[path = "add_values_tests_templates.rs"]
mod tests_templates;
