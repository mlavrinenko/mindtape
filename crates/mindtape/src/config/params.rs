//! `[params]` and `param()`: a pack's parameters, the `with` values an
//! `extends` entry sets for them, and the rewrite that writes each
//! `param("name")` call in the pack's expressions as the value's literal.
//!
//! [`crate::config::packs`] calls [`apply_params`] on each pack between
//! resolving it and taking its rule arrays out, so a rule's `when` and
//! `assert` reach the merge with the values already in them, and `[params]`
//! never reaches the merged config. A layer's own expressions have no
//! parameters to read: [`refuse_param_calls`] refuses a `param()` there.
//!
//! The rewrite skips string literals, the way
//! [`crate::filter::expr::join_kebab_names`] does, so `title ==
//! "param(\"x\")"` stays as written. It reaches only [`EXPRESSION_KEYS`]: a
//! rule's `message` is prose, not an expression.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::{ConfigError, quote};
use crate::problem::Problem;
use crate::suggest::Unknown;

/// One `extends` entry: a pack's name, or a table naming the pack and the
/// values of its parameters.
// `mt` reads the entries from the raw TOML before this type is built, through
// `packs_toml::read_extends`, so this type only describes them to the schema.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, schemars::JsonSchema)]
#[serde(untagged)]
pub enum PackEntry {
    /// A pack name, `<source>:<pack>`, or a path to a TOML pack file. Every
    /// parameter of the pack keeps its default.
    Name(String),
    /// A pack and the values of some of its parameters.
    With(PackWith),
}

/// An `extends` entry that sets a pack's parameters: `{ pack = "./team.toml",
/// with = { max-files = 8 } }`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackWith {
    /// The pack, named as in a bare entry.
    pub pack: String,
    /// A value for each parameter to change, keyed by the name the pack's
    /// `[params]` declares. A value takes the type of the parameter's
    /// default.
    #[serde(default)]
    pub with: BTreeMap<String, ParamValue>,
}

/// One parameter value: a string, a number or a boolean.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, schemars::JsonSchema)]
#[serde(untagged)]
pub enum ParamValue {
    /// `true` or `false`.
    Boolean(bool),
    /// A whole number.
    Integer(i64),
    /// A number with a fraction.
    Float(f64),
    /// A string, written into the expression as a quoted literal.
    String(String),
}

/// Every config key whose value is an expression, as TOML path steps. `*`
/// steps into every key of a table or every entry of an array.
const EXPRESSION_KEYS: [&[&str]; 18] = [
    &["tasks", "rules", "*", "when"],
    &["tasks", "rules", "*", "assert"],
    &["tasks", "body", "headers", "rules", "*", "when"],
    &["tasks", "body", "content", "rules", "*", "when"],
    &["commits", "rules", "*", "when"],
    &["commits", "rules", "*", "assert"],
    &["on", "flip", "*", "when"],
    &["views", "*", "filter"],
    &["tasks", "*", "required"],
    &["tasks", "fields", "*", "required"],
    &["tasks", "*", "values-when"],
    &["tasks", "fields", "*", "values-when"],
    &["tasks", "priority", "frameworks", "*", "score"],
    &[
        "tasks",
        "priority",
        "frameworks",
        "*",
        "category",
        "*",
        "when",
    ],
    &["ls", "rules", "*", "condition"],
    &["ls", "rules", "*", "filter"],
    &["ls", "columns", "*", "when"],
    &["rm", "default-filter"],
];

/// Take `[params]` out of `pack`, set each parameter `with` names, and write
/// every `param()` call in the pack's expressions as its value's literal.
/// `label` names the pack in an error.
///
/// # Errors
/// Returns [`ConfigError::Invalid`] for a parameter that isn't a string, a
/// number or a boolean, a `with` key the pack doesn't declare or of another
/// type than its default, and a `param()` call that names no parameter or
/// isn't one quoted name.
pub(super) fn apply_params(
    pack: &mut toml::Table,
    with: &toml::Table,
    label: &str,
) -> Result<(), ConfigError> {
    let values = resolve_values(pack.remove("params"), with, label)?;
    rewrite_expressions(pack, &mut |key, expr| {
        rewrite_calls(expr, |name| values.get(name).map(literal)).map_err(|error| match error {
            CallError::Malformed => ConfigError::invalid(format!(
                "extends: pack {} calls param() without one quoted name in {}",
                quote(label),
                quote(key)
            )),
            CallError::Unknown(name) => ConfigError::invalid(
                Problem::new(format!(
                    "extends: pack {} reads an undeclared parameter in {}: {}",
                    quote(label),
                    quote(key),
                    quote(&name)
                ))
                .with(Unknown::among(&name, values.keys().map(String::as_str))),
            ),
        })
    })
}

/// Refuse a `param()` call in a layer's own expressions, which no pack's
/// parameters reach.
///
/// # Errors
/// Returns [`ConfigError::Invalid`] naming the first expression key holding
/// one.
pub(super) fn refuse_param_calls(layer: &mut toml::Table) -> Result<(), ConfigError> {
    rewrite_expressions(layer, &mut |key, expr| {
        rewrite_calls(expr, |_| None).map_err(|error| {
            let message =
                format!("{key}: param() reads a pack's parameter, and this layer is no pack");
            ConfigError::invalid(match error {
                CallError::Unknown(name) => format!("{message}: {}", quote(&name)),
                CallError::Malformed => message,
            })
        })
    })
}

/// The pack's parameter defaults, each replaced by the value `with` sets.
fn resolve_values(
    params: Option<toml::Value>,
    with: &toml::Table,
    label: &str,
) -> Result<toml::Table, ConfigError> {
    let mut values = match params {
        None => toml::Table::new(),
        Some(toml::Value::Table(params)) => params,
        Some(_) => {
            return Err(ConfigError::invalid(format!(
                "extends: pack {} declares params as no table",
                quote(label)
            )));
        }
    };
    if let Some((name, _)) = values.iter().find(|(_, value)| !is_scalar(value)) {
        return Err(ConfigError::invalid(format!(
            "extends: pack {} parameter {} must be a string, a number or a boolean",
            quote(label),
            quote(name)
        )));
    }
    for (name, value) in with {
        let Some(default) = values.get(name) else {
            return Err(ConfigError::invalid(
                Problem::new(format!(
                    "extends: pack {} has no parameter: {}",
                    quote(label),
                    quote(name)
                ))
                .with(Unknown::among(name, values.keys().map(String::as_str))),
            ));
        };
        if type_name(default) != type_name(value) {
            return Err(ConfigError::invalid(format!(
                "extends: pack {} parameter {} takes {}, not {}",
                quote(label),
                quote(name),
                type_name(default),
                type_name(value)
            )));
        }
        values.insert(name.clone(), value.clone());
    }
    Ok(values)
}

fn is_scalar(value: &toml::Value) -> bool {
    matches!(
        value,
        toml::Value::String(_)
            | toml::Value::Integer(_)
            | toml::Value::Float(_)
            | toml::Value::Boolean(_)
    )
}

fn type_name(value: &toml::Value) -> &'static str {
    match value {
        toml::Value::String(_) => "a string",
        toml::Value::Integer(_) => "an integer",
        toml::Value::Float(_) => "a float",
        toml::Value::Boolean(_) => "a boolean",
        toml::Value::Datetime(_) => "a date",
        toml::Value::Array(_) => "an array",
        toml::Value::Table(_) => "a table",
    }
}

/// `value` as the filter language writes it: a string in double quotes with
/// `\` and `"` escaped, anything else as TOML prints it.
fn literal(value: &toml::Value) -> String {
    match value {
        toml::Value::String(text) => {
            format!("\"{}\"", text.replace('\\', r"\\").replace('"', "\\\""))
        }
        other => other.to_string(),
    }
}

/// What rewrites one expression: its dotted key and text in, the new text out.
type Rewrite<'f> = dyn FnMut(&str, &str) -> Result<String, ConfigError> + 'f;

/// Run `rewrite` over every string at one of [`EXPRESSION_KEYS`] in `table`.
fn rewrite_expressions(table: &mut toml::Table, rewrite: &mut Rewrite) -> Result<(), ConfigError> {
    EXPRESSION_KEYS.iter().try_for_each(|pattern| {
        let Some((step, rest)) = pattern.split_first() else {
            return Ok(());
        };
        visit_table(table, step, rest, "", rewrite)
    })
}

/// Visit each entry of `table` that `step` matches, `key` being the table's
/// own dotted key.
fn visit_table(
    table: &mut toml::Table,
    step: &str,
    rest: &[&str],
    key: &str,
    rewrite: &mut Rewrite,
) -> Result<(), ConfigError> {
    for (name, child) in table.iter_mut() {
        if step == "*" || name == step {
            let child_key = if key.is_empty() {
                name.clone()
            } else {
                format!("{key}.{name}")
            };
            visit(child, rest, &child_key, rewrite)?;
        }
    }
    Ok(())
}

/// Follow `pattern` down from `value`, at dotted key `key`, and rewrite the
/// string at its end.
fn visit(
    value: &mut toml::Value,
    pattern: &[&str],
    key: &str,
    rewrite: &mut Rewrite,
) -> Result<(), ConfigError> {
    let Some((step, rest)) = pattern.split_first() else {
        if let toml::Value::String(expr) = value {
            *expr = rewrite(key, expr)?;
        }
        return Ok(());
    };
    match value {
        toml::Value::Table(table) => visit_table(table, step, rest, key, rewrite),
        toml::Value::Array(items) if *step == "*" => {
            for (index, item) in items.iter_mut().enumerate() {
                visit(item, rest, &format!("{key}[{index}]"), rewrite)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Why a `param()` call could not be rewritten.
#[derive(Debug, PartialEq, Eq)]
enum CallError {
    /// Its argument is not one double-quoted name.
    Malformed,
    /// No parameter has this name.
    Unknown(String),
}

/// `expr` with each `param("name")` call outside a string literal replaced
/// by `lookup(name)`. `param` inside a longer name, such as `my-param`, is no
/// call, and neither is a `param` with no `(` after it.
fn rewrite_calls(expr: &str, lookup: impl Fn(&str) -> Option<String>) -> Result<String, CallError> {
    let mut out = String::with_capacity(expr.len());
    let mut rest = expr;
    let (mut in_string, mut escaped) = (false, false);
    // Whether the previous character continues a run of name characters.
    let mut in_run = false;
    while let Some(ch) = rest.chars().next() {
        if in_string {
            in_string = escaped || ch != '"';
            escaped = !escaped && ch == '\\';
        } else if !in_run && (ch.is_alphabetic() || ch == '_') {
            let (name, tail) = rest.split_at(name_len(rest));
            if name == "param"
                && let Some((argument, after)) = call_argument(tail)?
            {
                out.push_str(&lookup(&argument).ok_or(CallError::Unknown(argument))?);
                rest = after;
                in_run = false;
            } else {
                out.push_str(name);
                rest = tail;
                in_run = true;
            }
            continue;
        } else {
            in_string = ch == '"';
            in_run = is_name_char(ch);
        }
        out.push(ch);
        rest = rest.get(ch.len_utf8()..).unwrap_or_default();
    }
    Ok(out)
}

fn is_name_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '.'
}

/// The byte length of the name `text` starts with, a hyphen joining two
/// name characters as [`crate::filter::expr::join_kebab_names`] joins them.
fn name_len(text: &str) -> usize {
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        let joins = ch == '-'
            && chars
                .peek()
                .is_some_and(|(_, next)| next.is_alphanumeric() || *next == '_');
        if !is_name_char(ch) && !joins {
            return index;
        }
    }
    text.len()
}

/// The name a call's `("name")` passes, and the text after its `)`. `None`
/// when `tail` opens no call.
fn call_argument(tail: &str) -> Result<Option<(String, &str)>, CallError> {
    let Some(inner) = tail.trim_start().strip_prefix('(') else {
        return Ok(None);
    };
    let body = inner
        .trim_start()
        .strip_prefix('"')
        .ok_or(CallError::Malformed)?;
    let mut name = String::new();
    let mut escaped = false;
    for (index, ch) in body.char_indices() {
        match (escaped, ch) {
            (false, '\\') => escaped = true,
            (false, '"') => {
                let after = body.get(index + 1..).unwrap_or_default();
                let after = after.trim_start().strip_prefix(')');
                return after
                    .map(|after| Some((name, after)))
                    .ok_or(CallError::Malformed);
            }
            _ => {
                name.push(ch);
                escaped = false;
            }
        }
    }
    Err(CallError::Malformed)
}

#[cfg(test)]
#[path = "params_tests.rs"]
mod tests;
