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
pub struct CompiledRule {
    engine_rule: artifact_gate::Rule,
    when: Option<Filter>,
}

/// Errors from compiling header rules.
#[derive(Debug, Error)]
pub enum GateError {
    /// A path segment regex could not be compiled.
    #[error("invalid regex `{pattern}`: {error}")]
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
}

/// Compile header rules, pre-parsing `when` expressions and compiling regex segments.
///
/// # Errors
/// Returns [`GateError`] if any rule has an empty path, an invalid regex, or an
/// invalid `when` expression.
pub fn compile_rules(rules: &[HeaderRuleConfig]) -> Result<Vec<CompiledRule>, GateError> {
    rules.iter().map(compile_rule).collect()
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

fn compile_rule(rule: &HeaderRuleConfig) -> Result<CompiledRule, GateError> {
    if rule.path.is_empty() {
        return Err(GateError::EmptyPath);
    }
    let path = rule
        .path
        .iter()
        .map(compile_segment)
        .collect::<Result<_, _>>()?;
    let action = match rule.action {
        HeaderAction::Require => artifact_gate::Action::Require,
        HeaderAction::Forbid => artifact_gate::Action::Forbid,
    };
    let mut engine_rule = artifact_gate::Rule::new(path, action).map_err(|err| match err {
        artifact_gate::RuleError::EmptyPath => GateError::EmptyPath,
        artifact_gate::RuleError::Regex { pattern, error } => GateError::Regex { pattern, error },
    })?;
    if let Some(message) = &rule.message {
        engine_rule = engine_rule.with_message(message.clone());
    }
    let when = rule.when.as_deref().map(Filter::parse).transpose()?;
    Ok(CompiledRule { engine_rule, when })
}

fn compile_segment(seg: &HeaderPathSegment) -> Result<artifact_gate::Matcher, GateError> {
    match seg {
        HeaderPathSegment::Exact(text) => Ok(artifact_gate::Matcher::exact(text.clone())),
        HeaderPathSegment::Pattern { pattern, recursive } => {
            artifact_gate::Matcher::pattern(pattern, *recursive).map_err(|err| match err {
                artifact_gate::RuleError::EmptyPath => GateError::EmptyPath,
                artifact_gate::RuleError::Regex { pattern: p, error } => {
                    GateError::Regex { pattern: p, error }
                }
            })
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
    if !rule_applies(rule, task, config) {
        return;
    }
    let diagnostics = artifact_gate::run_rules(roots, std::slice::from_ref(&rule.engine_rule));
    for diag in diagnostics {
        out.push(diagnostic_to_violation(task, diag));
    }
}

fn diagnostic_to_violation(task: &Task, diag: artifact_gate::Diagnostic) -> Violation {
    let location = diag.line.map(|line| Location {
        file: task.file.clone(),
        line,
    });
    Violation {
        task_id: task.id.clone(),
        field: "body.headers".to_owned(),
        message: diag.message,
        location,
    }
}

fn rule_applies(rule: &CompiledRule, task: &Task, config: &Config) -> bool {
    let Some(filter) = &rule.when else {
        return true;
    };
    filter.matches(task, config).unwrap_or(false)
}

#[cfg(test)]
#[path = "gate_tests.rs"]
mod tests;
