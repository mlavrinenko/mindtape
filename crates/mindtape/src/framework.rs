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
//! (reusing `evalexpr`) and the validation that keeps an active framework's
//! achievable score within the configured scale. Built-ins ship as a baked-in
//! default config layer, deep-merged under any user definitions.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::config::PriorityConfig;
use crate::framework_eval::{eval_predicate, eval_score, parse_check, score_interval};
use crate::merge::deep_merge;
use crate::numeric::{InputValue, NumCell, NumValue, format_num};

/// The baked-in framework definitions, parsed from the bundled TOML.
const BUILTIN_FRAMEWORKS: &str = include_str!("frameworks.toml");

/// One framework definition: typed inputs plus either a scalar score formula or
/// a categorical classification.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Framework {
    /// Declared inputs, keyed by name (the names used in formulas/predicates).
    pub inputs: BTreeMap<String, Input>,
    /// Scalar score formula (evalexpr). Mutually exclusive with `categories`.
    pub score: Option<String>,
    /// Categorical classification. Mutually exclusive with `score`.
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

/// One categorical bucket: a predicate plus its representative score and display.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Category {
    /// evalexpr predicate over the inputs selecting this category.
    pub when: String,
    /// Representative score on the scale (drives banding and sort).
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

/// The framework-validation scope: the field label, framework name, and scale.
struct Scope<'a> {
    field: &'a str,
    name: &'a str,
    min: f64,
    max: f64,
}

impl Framework {
    /// Validate this framework definition against the active scale.
    fn validate(&self, scope: &Scope, out: &mut Vec<String>) {
        self.validate_inputs(scope, out);
        match (self.score.as_deref(), self.categories.is_empty()) {
            (Some(_), false) => out.push(format!(
                "{}: framework `{}` declares both a score and categories",
                scope.field, scope.name
            )),
            (None, true) => out.push(format!(
                "{}: framework `{}` declares neither a score nor categories",
                scope.field, scope.name
            )),
            (Some(formula), true) => self.validate_scalar(scope, formula, out),
            (None, false) => self.validate_categorical(scope, out),
        }
    }

    fn validate_inputs(&self, scope: &Scope, out: &mut Vec<String>) {
        for (name, input) in &self.inputs {
            match input.kind {
                InputKind::Number => {
                    if !matches!((input.min, input.max), (Some(lo), Some(hi)) if lo < hi) {
                        out.push(format!(
                            "{}: framework `{}` input `{name}` needs min < max",
                            scope.field, scope.name
                        ));
                    }
                }
                InputKind::Enum => {
                    if input.values.is_empty() {
                        out.push(format!(
                            "{}: framework `{}` enum input `{name}` needs values",
                            scope.field, scope.name
                        ));
                    }
                }
                InputKind::Bool => {}
            }
        }
    }

    fn validate_scalar(&self, scope: &Scope, formula: &str, out: &mut Vec<String>) {
        match score_interval(formula, &self.inputs) {
            Err(err) => out.push(format!("{}: framework `{}` {err}", scope.field, scope.name)),
            Ok(interval) => {
                const EPS: f64 = 1e-9;
                if interval.lo < scope.min - EPS || interval.hi > scope.max + EPS {
                    out.push(format!(
                        "{}: framework `{}` score range {}..{} exceeds the scale {}..{}",
                        scope.field,
                        scope.name,
                        format_num(interval.lo),
                        format_num(interval.hi),
                        format_num(scope.min),
                        format_num(scope.max),
                    ));
                }
            }
        }
    }

    fn validate_categorical(&self, scope: &Scope, out: &mut Vec<String>) {
        for category in &self.categories {
            if category.score < scope.min || category.score > scope.max {
                out.push(format!(
                    "{}: framework `{}` category `{}` score {} is outside the scale {}..{}",
                    scope.field,
                    scope.name,
                    category.label.as_deref().unwrap_or(""),
                    format_num(category.score),
                    format_num(scope.min),
                    format_num(scope.max),
                ));
            }
            if let Err(err) = parse_check(&category.when) {
                out.push(format!("{}: framework `{}` {err}", scope.field, scope.name));
            }
        }
        self.check_coverage(scope, out);
    }

    /// Verify the categories are exhaustive and disjoint over the boolean/enum
    /// input domain. Skipped when any input is numeric (not enumerable).
    fn check_coverage(&self, scope: &Scope, out: &mut Vec<String>) {
        if self.inputs.is_empty()
            || self
                .inputs
                .values()
                .any(|input| input.kind == InputKind::Number)
        {
            return;
        }
        for combo in combinations(&self.category_domains()) {
            let matched = self
                .categories
                .iter()
                .filter(|category| eval_predicate(&category.when, &combo).unwrap_or(false))
                .count();
            if matched != 1 {
                out.push(format!(
                    "{}: framework `{}` categories are not exhaustive+disjoint for {} ({matched} match)",
                    scope.field,
                    scope.name,
                    describe(&combo),
                ));
            }
        }
    }

    /// The enumerable value domain of each boolean/enum input.
    fn category_domains(&self) -> Vec<(String, Vec<InputValue>)> {
        self.inputs
            .iter()
            .filter_map(|(name, input)| match input.kind {
                InputKind::Bool => Some((
                    name.clone(),
                    vec![InputValue::Bool(false), InputValue::Bool(true)],
                )),
                InputKind::Enum => Some((
                    name.clone(),
                    input
                        .values
                        .iter()
                        .map(|value| InputValue::Text(value.clone()))
                        .collect(),
                )),
                InputKind::Number => None,
            })
            .collect()
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

    /// Validate the numeric field plus the active framework definition.
    #[must_use]
    pub fn validate(&self, field: &str) -> Vec<String> {
        let mut out = self.field.validate(field);
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

/// Cartesian product of each input's value domain into concrete input maps.
fn combinations(domains: &[(String, Vec<InputValue>)]) -> Vec<BTreeMap<String, InputValue>> {
    let mut acc = vec![BTreeMap::new()];
    for (name, values) in domains {
        let mut next = Vec::new();
        for base in &acc {
            for value in values {
                let mut combo = base.clone();
                combo.insert(name.clone(), value.clone());
                next.push(combo);
            }
        }
        acc = next;
    }
    acc
}

/// A human-readable rendering of an input combination, for error messages.
fn describe(combo: &BTreeMap<String, InputValue>) -> String {
    combo
        .iter()
        .map(|(name, value)| format!("{name}={}", display_input(value)))
        .collect::<Vec<_>>()
        .join(", ")
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

fn display_input(value: &InputValue) -> String {
    match value {
        InputValue::Num(num) => format_num(*num),
        InputValue::Bool(flag) => flag.to_string(),
        InputValue::Text(text) => text.clone(),
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
