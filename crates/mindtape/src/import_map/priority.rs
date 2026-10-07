//! `priority:`/`difficulty:` policy mapping: reverse the canonical schema's
//! framework/scalar shape into [`NumValue`], then report scale and
//! strict-framework problems via the same pure checks `validate/mod.rs` uses
//! ([`crate::framework::Framework::check_input_values`],
//! [`crate::config::PriorityConfig::resolve`], [`NumericField::resolve`]).

use std::collections::BTreeMap;

use serde_json::Value;

use crate::config::{Config, PriorityConfig};
use crate::model::canonical::{CanonicalNum, CanonicalPriority};
use crate::model::numeric::{InputValue, NumValue, NumericField, format_num};

use super::report::ReportEntry;

/// Reverse a canonical `priority` into its authored [`NumValue`] form,
/// reporting strict-framework and scale violations. `score` is read-side only
/// and ignored (see the module doc's "Priority parity" section in
/// `model/canonical.rs`).
pub(crate) fn map_priority(
    canonical: Option<&CanonicalPriority>,
    config: &Config,
    out: &mut Vec<ReportEntry>,
) -> Option<NumValue> {
    let priority_cfg = &config.tasks.priority;
    let value = match canonical? {
        CanonicalPriority::Num(num) => NumValue::Num(*num),
        CanonicalPriority::Label(label) => NumValue::Label(label.clone()),
        CanonicalPriority::Framework {
            framework, inputs, ..
        } => NumValue::Framework {
            name: framework.clone(),
            inputs: convert_inputs(inputs, out),
        },
    };
    check_priority(&value, priority_cfg, out);
    Some(value)
}

/// Reverse a canonical `difficulty` into its authored [`NumValue`] form
/// (never a framework — the canonical schema never authors one, see
/// `model/canonical.rs::num_to_canonical`), reporting scale violations.
pub(crate) fn map_difficulty(
    canonical: Option<&CanonicalNum>,
    config: &Config,
    out: &mut Vec<ReportEntry>,
) -> Option<NumValue> {
    let value = match canonical? {
        CanonicalNum::Num(num) => NumValue::Num(*num),
        CanonicalNum::Label(label) => NumValue::Label(label.clone()),
    };
    check_scale(&value, "difficulty", &config.tasks.difficulty, out);
    Some(value)
}

fn convert_inputs(
    inputs: &BTreeMap<String, Value>,
    out: &mut Vec<ReportEntry>,
) -> BTreeMap<String, InputValue> {
    let mut converted = BTreeMap::new();
    for (key, value) in inputs {
        match json_to_input_value(value) {
            Some(input) => {
                converted.insert(key.clone(), input);
            }
            None => out.push(ReportEntry::violated(
                "priority",
                format!("input `{key}` has an unsupported JSON type; dropped"),
            )),
        }
    }
    converted
}

fn json_to_input_value(value: &Value) -> Option<InputValue> {
    match value {
        Value::Number(_) => value.as_f64().map(InputValue::Num),
        Value::Bool(flag) => Some(InputValue::Bool(*flag)),
        Value::String(text) => Some(InputValue::Text(text.clone())),
        Value::Null | Value::Array(_) | Value::Object(_) => None,
    }
}

fn check_priority(value: &NumValue, config: &PriorityConfig, out: &mut Vec<ReportEntry>) {
    match value {
        NumValue::Framework { name, inputs } => check_framework(value, name, inputs, config, out),
        NumValue::Num(_) | NumValue::Label(_) => check_manual(value, config, out),
    }
}

fn check_framework(
    value: &NumValue,
    name: &str,
    inputs: &BTreeMap<String, InputValue>,
    config: &PriorityConfig,
    out: &mut Vec<ReportEntry>,
) {
    match config.frameworks.get(name) {
        None => out.push(ReportEntry::violated(
            "priority",
            format!("unknown priority framework `{name}`"),
        )),
        Some(framework) => {
            for problem in framework.check_input_values(inputs) {
                out.push(ReportEntry::violated("priority", problem));
            }
        }
    }
    if let Some(score) = config.resolve(value) {
        push_if_out_of_scale(score, "priority", &config.field, out);
    }
}

fn check_manual(value: &NumValue, config: &PriorityConfig, out: &mut Vec<ReportEntry>) {
    if config.framework.is_some() && !config.allow_manual {
        out.push(ReportEntry::violated(
            "priority",
            format!(
                "`{}` is a manual priority but the project requires its active framework",
                value.as_display()
            ),
        ));
        return;
    }
    check_scale(value, "priority", &config.field, out);
}

fn check_scale(value: &NumValue, field: &str, scale: &NumericField, out: &mut Vec<ReportEntry>) {
    match scale.resolve(value) {
        None => out.push(ReportEntry::violated(
            field,
            format!("`{}` is not a known {field} value", value.as_display()),
        )),
        Some(point) => push_if_out_of_scale(point, field, scale, out),
    }
}

fn push_if_out_of_scale(point: f64, field: &str, scale: &NumericField, out: &mut Vec<ReportEntry>) {
    if point < scale.min || point > scale.max {
        out.push(ReportEntry::violated(
            field,
            format!(
                "{} is outside the {field} scale {}..{}",
                format_num(point),
                format_num(scale.min),
                format_num(scale.max),
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{Config, PriorityConfig, TasksConfig};
    use crate::model::canonical::{CanonicalNum, CanonicalPriority};
    use crate::model::numeric::{InputValue, NumValue};

    #[test]
    fn no_framework_with_disallowed_manual_is_clean() {
        let config = Config {
            tasks: TasksConfig {
                priority: PriorityConfig {
                    framework: None,
                    allow_manual: false,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let mut out = Vec::new();
        let result = super::map_priority(Some(&CanonicalPriority::Num(5.0)), &config, &mut out);
        assert_eq!(result, Some(NumValue::Num(5.0)));
        assert!(out.is_empty(), "expected no report entries, got {out:?}",);
    }

    #[test]
    fn difficulty_at_min_is_in_scale() {
        let config = Config::default();
        let mut out = Vec::new();
        let result = super::map_difficulty(Some(&CanonicalNum::Num(0.0)), &config, &mut out);
        assert_eq!(result, Some(NumValue::Num(0.0)));
        assert!(
            out.is_empty(),
            "expected no report entries for difficulty at min, got {out:?}",
        );
    }

    #[test]
    fn difficulty_at_max_is_in_scale() {
        let config = Config::default();
        let mut out = Vec::new();
        let result = super::map_difficulty(Some(&CanonicalNum::Num(9.0)), &config, &mut out);
        assert_eq!(result, Some(NumValue::Num(9.0)));
        assert!(
            out.is_empty(),
            "expected no report entries for difficulty at max, got {out:?}",
        );
    }

    #[test]
    fn json_number_converts_to_num() {
        assert_eq!(
            super::json_to_input_value(&serde_json::json!(2.5)),
            Some(InputValue::Num(2.5)),
        );
        assert_eq!(
            super::json_to_input_value(&serde_json::json!(8)),
            Some(InputValue::Num(8.0)),
        );
    }

    #[test]
    fn json_bool_converts_to_bool() {
        assert_eq!(
            super::json_to_input_value(&serde_json::json!(true)),
            Some(InputValue::Bool(true)),
        );
        assert_eq!(
            super::json_to_input_value(&serde_json::json!(false)),
            Some(InputValue::Bool(false)),
        );
    }

    #[test]
    fn json_string_converts_to_text() {
        assert_eq!(
            super::json_to_input_value(&serde_json::json!("high")),
            Some(InputValue::Text("high".to_owned())),
        );
    }

    #[test]
    fn json_null_array_object_are_dropped() {
        assert_eq!(super::json_to_input_value(&serde_json::json!(null)), None);
        assert_eq!(super::json_to_input_value(&serde_json::json!([])), None);
        assert_eq!(super::json_to_input_value(&serde_json::json!({})), None);
    }
}
