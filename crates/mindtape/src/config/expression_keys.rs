//! Every config key whose value is an expression, read from the config
//! schema once.
//!
//! A config struct tags each expression field where it declares it, with
//! `#[schemars(extend("x-expression" = "<language>"))]`. The language is
//! `filter` for the task filter language, `commit` for commit rules,
//! `condition` for the `[[ls.rules]]` and `[ls.columns]` grammar, and
//! `formula` for a priority framework's `score` and category `when`.
//! [`expression_keys`] walks the schema of [`Config`] and returns the TOML
//! path of each tagged field, which [`crate::config::params`] rewrites
//! `param()` calls in.

use std::sync::OnceLock;

use serde_json::{Map, Value};

use crate::config::Config;

/// The schema keyword an expression field carries, naming its language.
pub(crate) const EXPRESSION_TAG: &str = "x-expression";

/// One expression key of the config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpressionKey {
    /// TOML path steps from the config root. `*` steps into every key of a
    /// table or every entry of an array.
    pub(crate) path: Vec<String>,
    /// Whether the key's language reads `a-b` as one name, as the filter and
    /// commit languages do. A formula or a condition reads it as `a - b`.
    pub(crate) joins_kebab: bool,
}

/// Every expression key of [`Config`], in schema order.
pub(crate) fn expression_keys() -> &'static [ExpressionKey] {
    static KEYS: OnceLock<Vec<ExpressionKey>> = OnceLock::new();
    KEYS.get_or_init(|| {
        let schema = schemars::schema_for!(Config);
        let mut keys = Vec::new();
        walk_schema(schema.as_value(), &mut |path, node| {
            if let Some(language) = node.get(EXPRESSION_TAG).and_then(Value::as_str) {
                let key = ExpressionKey {
                    path: path.to_vec(),
                    joins_kebab: matches!(language, "filter" | "commit"),
                };
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        });
        keys
    })
}

/// What [`walk_schema`] calls on each node: the TOML path it describes, and
/// the node.
pub(crate) type Visit<'f> = dyn FnMut(&[String], &Map<String, Value>) + 'f;

/// Call `visit` on every schema node under `root`, with the TOML path it
/// describes. A `$ref` is followed into `$defs`, once per path, so a
/// recursive type ends.
pub(crate) fn walk_schema(root: &Value, visit: &mut Visit) {
    walk(root, root, &mut Vec::new(), &mut Vec::new(), visit);
}

fn walk(
    root: &Value,
    node: &Value,
    path: &mut Vec<String>,
    seen: &mut Vec<String>,
    visit: &mut Visit,
) {
    let Some(map) = node.as_object() else {
        return;
    };
    visit(path, map);
    if let Some(name) = map
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|reference| reference.strip_prefix("#/$defs/"))
        && !seen.iter().any(|done| done == name)
        && let Some(def) = root.get("$defs").and_then(|defs| defs.get(name))
    {
        seen.push(name.to_owned());
        walk(root, def, path, seen, visit);
        seen.pop();
    }
    for keyword in ["anyOf", "oneOf", "allOf"] {
        for sub in map
            .get(keyword)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            walk(root, sub, path, seen, visit);
        }
    }
    for (name, sub) in map
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        path.push(name.clone());
        walk(root, sub, path, seen, visit);
        path.pop();
    }
    for keyword in ["items", "additionalProperties"] {
        if let Some(sub) = map.get(keyword).filter(|sub| sub.is_object()) {
            path.push("*".to_owned());
            walk(root, sub, path, seen, visit);
            path.pop();
        }
    }
}

#[cfg(test)]
#[path = "expression_keys_tests.rs"]
mod tests;
