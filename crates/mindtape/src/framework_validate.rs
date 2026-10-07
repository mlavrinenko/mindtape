//! Validation of framework definitions against the active scale and the Typst
//! source they generate. Split out of `framework` so both modules fit their
//! file-size caps; the Rust layer keeps the evaluator, this module keeps every
//! config-load-time rejection.

use std::collections::BTreeMap;

use typst_syntax::is_ident;

use crate::framework::{Framework, InputKind};
use crate::framework_eval::{eval_predicate, parse_check, score_interval};
use crate::model::numeric::{InputValue, format_num};

/// The framework-validation scope: the field label, framework name, and scale.
pub(crate) struct Scope<'a> {
    pub(crate) field: &'a str,
    pub(crate) name: &'a str,
    pub(crate) min: f64,
    pub(crate) max: f64,
}

/// Typst's reserved words: lexing one yields a keyword token, not an
/// identifier, so a named argument cannot carry it (the parser demands an
/// `Ident` there). Mirrors the private keyword table in typst-syntax.
const TYPST_KEYWORDS: &[&str] = &[
    "none", "auto", "true", "false", "not", "and", "or", "let", "set", "show", "context", "if",
    "else", "for", "in", "while", "break", "continue", "return", "import", "include", "as",
];

/// Whether `name` is a name a Typst named argument can carry: an identifier
/// that is neither a keyword nor the bare underscore token (`_` lexes as
/// `SyntaxKind::Underscore`, not `Ident`).
fn is_typst_ident(name: &str) -> bool {
    is_ident(name) && name != "_" && !TYPST_KEYWORDS.contains(&name)
}

impl Framework {
    /// Validate every declared input name is a name a Typst named argument can
    /// carry. Input names reach the DSL twice — as named args in
    /// `framework("f", input: …)` calls and as `--set` keys — and an invalid
    /// name otherwise surfaces only after `mt add` writes the task, as an eval
    /// error. Runs for every declared framework, active or not.
    pub(crate) fn validate_input_names(&self, field: &str, name: &str, out: &mut Vec<String>) {
        for input in self.inputs.keys() {
            if !is_typst_ident(input) {
                out.push(format!(
                    "{field}: framework `{name}` input `{input}` is not a valid typst identifier"
                ));
            }
        }
    }

    /// Validate this framework definition against the active scale.
    pub(crate) fn validate(&self, scope: &Scope, out: &mut Vec<String>) {
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

fn display_input(value: &InputValue) -> String {
    match value {
        InputValue::Num(num) => format_num(*num),
        InputValue::Bool(flag) => flag.to_string(),
        InputValue::Text(text) => text.clone(),
    }
}

#[cfg(test)]
#[path = "framework_validate_tests.rs"]
mod tests;
