//! Config-driven body-header gating: rule compilation and heading-tree matching.
//!
//! Rules are declared in `[[tasks.body.headers.rules]]` in `.mindtape/config.toml`.
//! Compile them once with [`compile_rules`], then call [`check_headers`] once per
//! task's source text.

use thiserror::Error;
use typst_cst::{HeadingNode, parse_headings, tree};

use crate::config::{Config, HeaderAction, HeaderPathSegment, HeaderRuleConfig};
use crate::filter::{Filter, FilterError};
use crate::model::Task;
use crate::validate::{Location, Violation};

/// A compiled, ready-to-run header rule (pre-parsed filter, pre-compiled regex,
/// pre-baked diagnostic message).
#[derive(Debug)]
pub struct CompiledRule {
    engine_rule: artifact_gate::Rule,
    when: Option<Filter>,
    /// This rule's name if it has one, else its position — see
    /// [`HeaderRuleConfig::label`].
    label: String,
}

/// Errors from compiling header rules.
#[derive(Debug, Error)]
pub enum GateError {
    /// A path segment regex could not be compiled.
    #[error("invalid regex '{pattern}': {error}")]
    Regex {
        /// The offending pattern.
        pattern: String,
        /// Compiler error message.
        error: String,
    },
    /// A `when` expression could not be parsed.
    #[error("invalid `when` expression: {0}")]
    Filter(#[from] FilterError),
    /// A rule carries an empty path.
    #[error("rule path must not be empty")]
    EmptyPath,
    /// A rule names a field that its action does not accept.
    #[error("action '{action}' does not accept: {fields}")]
    UnexpectedFields {
        /// The rule's action.
        action: String,
        /// Comma-joined names of the offending fields.
        fields: String,
    },
}

/// Compile header rules, pre-parsing `when` expressions and compiling regex segments.
///
/// # Errors
/// Returns [`GateError`] if any rule has an empty path, an invalid regex, or an
/// invalid `when` expression.
pub fn compile_rules(rules: &[HeaderRuleConfig]) -> Result<Vec<CompiledRule>, GateError> {
    rules
        .iter()
        .enumerate()
        .map(|(index, rule)| compile_rule(rule, index))
        .collect()
}

/// Run compiled header rules against a task's source text.
///
/// Parses the heading structure with [`typst_cst`] then evaluates every rule.
/// `require` violations carry no location; `forbid` violations carry the location
/// of the offending heading.
#[must_use]
pub fn check_headers(
    task: &Task,
    source: &str,
    rules: &[CompiledRule],
    config: &Config,
) -> Vec<Violation> {
    let nodes = heading_nodes_to_gate(tree(parse_headings(source)));
    check_with_nodes(task, &nodes, rules, config)
}

fn compile_rule(rule: &HeaderRuleConfig, index: usize) -> Result<CompiledRule, GateError> {
    let mut engine_rule = match rule.action {
        HeaderAction::Require | HeaderAction::Forbid | HeaderAction::Unique => {
            ensure_empty(
                "require/forbid/unique",
                &[
                    ("before", &rule.before),
                    ("after", &rule.after),
                    ("scope", &rule.scope),
                    ("allowed", &rule.allowed),
                ],
            )?;
            let action = match rule.action {
                HeaderAction::Require => artifact_gate::Action::Require,
                HeaderAction::Forbid => artifact_gate::Action::Forbid,
                _ => artifact_gate::Action::Unique,
            };
            artifact_gate::Rule::path(compile_path(&rule.path)?, action).map_err(map_rule_error)?
        }
        HeaderAction::Order => {
            ensure_empty(
                "order",
                &[
                    ("path", &rule.path),
                    ("scope", &rule.scope),
                    ("allowed", &rule.allowed),
                ],
            )?;
            artifact_gate::Rule::order(compile_path(&rule.before)?, compile_path(&rule.after)?)
                .map_err(map_rule_error)?
        }
        HeaderAction::Allowlist => {
            ensure_empty(
                "allowlist",
                &[
                    ("path", &rule.path),
                    ("before", &rule.before),
                    ("after", &rule.after),
                ],
            )?;
            let scope = if rule.scope.is_empty() {
                None
            } else {
                Some(compile_path(&rule.scope)?)
            };
            artifact_gate::Rule::allow(scope, compile_path(&rule.allowed)?)
                .map_err(map_rule_error)?
        }
    };
    if let Some(message) = &rule.message {
        engine_rule = engine_rule.with_message(message.clone());
    }
    let when = rule.when.as_deref().map(Filter::parse).transpose()?;
    Ok(CompiledRule {
        engine_rule,
        when,
        label: rule.label(index),
    })
}

/// Compile a heading path into artifact-gate matchers, shared with the content
/// gate.
pub(crate) fn compile_path(
    segments: &[HeaderPathSegment],
) -> Result<Vec<artifact_gate::Matcher>, GateError> {
    segments.iter().map(compile_segment).collect()
}

fn ensure_empty(action: &str, fields: &[(&str, &Vec<HeaderPathSegment>)]) -> Result<(), GateError> {
    let bad: Vec<&str> = fields
        .iter()
        .filter(|(_, segs)| !segs.is_empty())
        .map(|(name, _)| *name)
        .collect();
    if bad.is_empty() {
        Ok(())
    } else {
        Err(GateError::UnexpectedFields {
            action: action.to_owned(),
            fields: bad.join(", "),
        })
    }
}

fn map_rule_error(err: artifact_gate::RuleError) -> GateError {
    match err {
        artifact_gate::RuleError::EmptyPath => GateError::EmptyPath,
        artifact_gate::RuleError::Regex { pattern, error } => GateError::Regex { pattern, error },
    }
}

fn compile_segment(seg: &HeaderPathSegment) -> Result<artifact_gate::Matcher, GateError> {
    match seg {
        HeaderPathSegment::Exact(text) => Ok(artifact_gate::Matcher::exact(text.clone())),
        HeaderPathSegment::Pattern { pattern, recursive } => {
            artifact_gate::Matcher::pattern(pattern, *recursive).map_err(map_rule_error)
        }
    }
}

fn heading_nodes_to_gate(nodes: Vec<HeadingNode>) -> Vec<artifact_gate::Node> {
    nodes
        .into_iter()
        .map(|n| artifact_gate::Node {
            kind: "heading".to_owned(),
            text: n.heading.text,
            line: n.heading.line,
            children: heading_nodes_to_gate(n.children),
        })
        .collect()
}

fn check_with_nodes(
    task: &Task,
    roots: &[artifact_gate::Node],
    rules: &[CompiledRule],
    config: &Config,
) -> Vec<Violation> {
    let mut out = Vec::new();
    for rule in rules {
        apply_rule(task, roots, rule, config, &mut out);
    }
    out
}

fn apply_rule(
    task: &Task,
    roots: &[artifact_gate::Node],
    rule: &CompiledRule,
    config: &Config,
    out: &mut Vec<Violation>,
) {
    if !crate::body::applies(rule.when.as_ref(), task, config) {
        return;
    }
    let diagnostics = artifact_gate::run_rules(roots, std::slice::from_ref(&rule.engine_rule));
    for diag in diagnostics {
        out.push(diagnostic_to_violation(task, diag, rule));
    }
}

fn diagnostic_to_violation(
    task: &Task,
    diag: artifact_gate::Diagnostic,
    rule: &CompiledRule,
) -> Violation {
    let location = diag.line.map_or_else(
        || task.marker_location(),
        |line| Location::at(task.file.clone(), line),
    );
    crate::body::body_violation(
        task,
        &rule.label,
        diag.message,
        location,
        rule.when.as_ref(),
    )
}

#[cfg(test)]
#[path = "gate_tests.rs"]
mod tests;
