//! Priority scoring frameworks: config-defined inputs and evalexpr scoring.
//!
//! A framework turns typed, bounded inputs into a numeric priority score. Two
//! shapes share one mechanism:
//!
//! - scalar — a `score` formula (evalexpr) over the inputs; the score feeds the
//!   `[tasks.priority]` bands (e.g. ICE, RICE, WSJF).
//! - categorical — a `category` list, each with a `when` predicate, a
//!   representative `score`, and display glyph/colour; categories must be
//!   exhaustive and disjoint over the boolean/enum input domain (e.g.
//!   Eisenhower, `MoSCoW`).
//!
//! Frameworks live entirely in config data. Rust supplies only the evaluator
//! (reusing `evalexpr`); the config-load-time validation lives in
//! [`crate::framework_validate`]. Built-ins ship as a baked-in default config
//! layer, deep-merged under any user definitions.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::config::PriorityConfig;
use crate::config::merge::deep_merge;
use crate::framework_eval::{eval_predicate, eval_score};
use crate::framework_validate::Scope;
use crate::numeric::{InputValue, NumCell, NumValue, format_num};

/// The baked-in framework definitions, parsed from the bundled TOML.
const BUILTIN_FRAMEWORKS: &str = include_str!("frameworks.toml");

/// One scoring framework: typed inputs, plus either a `score` formula or a
/// list of categories.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Framework {
    /// Inputs by name. `score` and each category's `when` refer to them by
    /// these names.
    pub inputs: BTreeMap<String, Input>,
    /// Arithmetic expression over the inputs that computes the score, such as
    /// `impact * confidence * ease`. Cannot be combined with `category`.
    pub score: Option<String>,
    /// Categories to classify a task into. For any input values, exactly one
    /// category's `when` must be true. Cannot be combined with `score`.
    #[serde(rename = "category")]
    pub categories: Vec<Category>,
}

/// One declared input: its type and bounds (numbers) or allowed values (enums).
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    /// Input type.
    #[serde(rename = "type")]
    pub kind: InputKind,
    /// Inclusive lower bound (number inputs).
    pub min: Option<f64>,
    /// Inclusive upper bound (number inputs).
    pub max: Option<f64>,
    /// Allowed values (enum inputs).
    pub values: Vec<String>,
}

/// The type of a framework input.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum InputKind {
    /// A bounded number.
    #[default]
    Number,
    /// A boolean.
    Bool,
    /// A string from a fixed set of values.
    Enum,
}

/// One category: the expression that selects it, its score and its display.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Category {
    /// Boolean expression over the inputs. A task falls in this category
    /// when it is true.
    pub when: String,
    /// Score of a task in this category, used for bands and sorting.
    pub score: f64,
    /// Display label; falls back to empty when absent.
    pub label: Option<String>,
    /// Optional decorative glyph.
    pub glyph: Option<String>,
    /// Optional colour: a named ANSI colour or `#RRGGBB`.
    pub color: Option<String>,
}

impl Framework {
    /// Whether this framework classifies into categories (vs a scalar score).
    #[must_use]
    pub fn is_categorical(&self) -> bool {
        !self.categories.is_empty()
    }

    /// The numeric score for a set of inputs: a scalar formula evaluates; a
    /// categorical framework yields its matched category's score.
    ///
    /// # Errors
    /// Returns a message when evaluation fails or no category matches.
    pub fn score_of(&self, inputs: &BTreeMap<String, InputValue>) -> Result<f64, String> {
        if self.is_categorical() {
            return self.match_category(inputs).map(|category| category.score);
        }
        let formula = self
            .score
            .as_deref()
            .ok_or_else(|| "framework has no score formula".to_owned())?;
        eval_score(formula, inputs)
    }

    /// The display cell for a categorical framework's inputs.
    fn category_cell(&self, inputs: &BTreeMap<String, InputValue>) -> Option<NumCell> {
        let category = self.match_category(inputs).ok()?;
        Some(NumCell {
            label: category.label.clone().unwrap_or_default(),
            glyph: category.glyph.clone(),
            color: category.color.clone(),
        })
    }

    /// Validate provided input values against the declared inputs: each declared
    /// input must be present and type/bounds-correct; no unknown inputs allowed.
    #[must_use]
    pub fn check_input_values(&self, inputs: &BTreeMap<String, InputValue>) -> Vec<String> {
        let mut out = Vec::new();
        for (name, input) in &self.inputs {
            match inputs.get(name) {
                None => out.push(format!("missing input `{name}`")),
                Some(value) => check_input_value(name, input, value, &mut out),
            }
        }
        for name in inputs.keys() {
            if !self.inputs.contains_key(name) {
                out.push(format!("unknown input `{name}`"));
            }
        }
        out
    }

    fn match_category(&self, inputs: &BTreeMap<String, InputValue>) -> Result<&Category, String> {
        for category in &self.categories {
            if eval_predicate(&category.when, inputs)? {
                return Ok(category);
            }
        }
        Err("no category matched the inputs".to_owned())
    }
}

impl PriorityConfig {
    /// The active framework, if one is selected and present in the registry.
    #[must_use]
    pub fn active_framework(&self) -> Option<&Framework> {
        self.frameworks.get(self.framework.as_deref()?)
    }

    /// Resolve a written value to its numeric score on the scale, if resolvable.
    /// Frameworks evaluate through evalexpr; numbers and labels delegate to the
    /// numeric field. Best-effort: `None` when unresolvable (used by sort/filter).
    #[must_use]
    pub fn resolve(&self, value: &NumValue) -> Option<f64> {
        match value {
            NumValue::Framework { name, inputs } => {
                self.frameworks.get(name)?.score_of(inputs).ok()
            }
            other => self.field.resolve(other),
        }
    }

    /// Resolve and classify a written value into its display cell. A categorical
    /// framework yields its category cell; everything else classifies the score
    /// through the numeric field's bands.
    #[must_use]
    pub fn cell(&self, value: &NumValue) -> Option<NumCell> {
        match value {
            NumValue::Framework { name, inputs } => {
                let framework = self.frameworks.get(name)?;
                if framework.is_categorical() {
                    framework.category_cell(inputs)
                } else {
                    self.field.classify(framework.score_of(inputs).ok()?)
                }
            }
            other => self.field.cell(other),
        }
    }

    /// Evaluate a framework value into its score, with a descriptive error.
    ///
    /// # Errors
    /// Returns a message when the framework is unknown or evaluation fails.
    pub fn evaluate(
        &self,
        name: &str,
        inputs: &BTreeMap<String, InputValue>,
    ) -> Result<f64, String> {
        self.frameworks
            .get(name)
            .ok_or_else(|| format!("unknown framework `{name}`"))?
            .score_of(inputs)
    }

    /// Validate the numeric field plus every declared framework's input names
    /// and the active framework definition.
    #[must_use]
    pub fn validate(&self, field: &str) -> Vec<String> {
        let mut out = self.field.validate(field);
        for (name, framework) in &self.frameworks {
            framework.validate_input_names(field, name, &mut out);
        }
        if let Some(name) = &self.framework {
            match self.frameworks.get(name) {
                None => out.push(format!("{field}: unknown framework `{name}`")),
                Some(framework) => {
                    let scope = Scope {
                        field,
                        name,
                        min: self.field.min,
                        max: self.field.max,
                    };
                    framework.validate(&scope, &mut out);
                }
            }
        }
        out
    }
}

fn check_input_value(name: &str, input: &Input, value: &InputValue, out: &mut Vec<String>) {
    match (input.kind, value) {
        (InputKind::Number, InputValue::Num(num)) => {
            let below = input.min.is_some_and(|min| *num < min);
            let above = input.max.is_some_and(|max| *num > max);
            if below || above {
                out.push(format!(
                    "input `{name}` value {} is out of bounds",
                    format_num(*num)
                ));
            }
        }
        (InputKind::Bool, InputValue::Bool(_)) => {}
        (InputKind::Enum, InputValue::Text(text)) => {
            if !input.values.iter().any(|allowed| allowed == text) {
                out.push(format!("input `{name}` value `{text}` is not allowed"));
            }
        }
        _ => out.push(format!("input `{name}` has the wrong type")),
    }
}

// --- Built-in framework layer ------------------------------------------------

/// Deep-merge the baked-in built-in frameworks under any user-declared ones at
/// `[tasks.priority.frameworks]`, mutating the parsed config document in place.
///
/// The user's table wins field-by-field, so a project can activate a built-in
/// with no boilerplate yet still override (say) one framework's `score`.
pub(crate) fn merge_builtin_frameworks(doc: &mut toml::Table) {
    let Ok(builtins) = toml::from_str::<toml::Table>(BUILTIN_FRAMEWORKS) else {
        return;
    };
    let user = take_frameworks(doc);
    let mut merged = toml::Value::Table(builtins);
    if let Some(user) = user {
        deep_merge(&mut merged, toml::Value::Table(user));
    }
    set_frameworks(doc, merged);
}

fn take_frameworks(doc: &mut toml::Table) -> Option<toml::Table> {
    let tasks = doc.get_mut("tasks")?.as_table_mut()?;
    let priority = tasks.get_mut("priority")?.as_table_mut()?;
    match priority.remove("frameworks")? {
        toml::Value::Table(table) => Some(table),
        _ => None,
    }
}

fn set_frameworks(doc: &mut toml::Table, frameworks: toml::Value) {
    let Some(tasks) = ensure_table(doc, "tasks") else {
        return;
    };
    let Some(priority) = ensure_table(tasks, "priority") else {
        return;
    };
    priority.insert("frameworks".to_owned(), frameworks);
}

fn ensure_table<'a>(parent: &'a mut toml::Table, key: &str) -> Option<&'a mut toml::Table> {
    parent
        .entry(key)
        .or_insert_with(|| toml::Value::Table(toml::Table::new()))
        .as_table_mut()
}

#[cfg(test)]
#[path = "framework_tests.rs"]
mod tests;
