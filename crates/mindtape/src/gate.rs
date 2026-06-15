//! Config-driven body-header gating: rule compilation and heading-tree matching.
//!
//! Rules are declared in `[[tasks.body.headers.rules]]` in `.mindtape/config.toml`.
//! Compile them once with [`compile_rules`], then call [`check_headers`] once per
//! task's source text.

use regex::Regex;
use thiserror::Error;
use typst_cst::{HeadingNode, parse_headings, tree};

use crate::config::{HeaderAction, HeaderPathSegment, HeaderRuleConfig};
use crate::filter::{Filter, FilterError};
use crate::model::Task;
use crate::validate::{Location, Violation};

/// A compiled, ready-to-run header rule (pre-parsed filter, pre-compiled regex).
pub struct CompiledRule {
    path: Vec<CompiledSegment>,
    action: HeaderAction,
    when: Option<Filter>,
    message: Option<String>,
}

enum CompiledSegment {
    Exact(String),
    Pattern { re: Box<Regex>, recursive: bool },
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
pub fn check_headers(task: &Task, source: &str, rules: &[CompiledRule]) -> Vec<Violation> {
    let nodes = tree(parse_headings(source));
    check_with_nodes(task, &nodes, rules)
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
    let when = rule.when.as_deref().map(Filter::parse).transpose()?;
    Ok(CompiledRule {
        path,
        action: rule.action.clone(),
        when,
        message: rule.message.clone(),
    })
}

fn compile_segment(seg: &HeaderPathSegment) -> Result<CompiledSegment, GateError> {
    Ok(match seg {
        HeaderPathSegment::Exact(text) => CompiledSegment::Exact(text.clone()),
        HeaderPathSegment::Pattern { pattern, recursive } => CompiledSegment::Pattern {
            re: Box::new(Regex::new(pattern).map_err(|err| GateError::Regex {
                pattern: pattern.clone(),
                error: err.to_string(),
            })?),
            recursive: *recursive,
        },
    })
}

fn check_with_nodes(task: &Task, roots: &[HeadingNode], rules: &[CompiledRule]) -> Vec<Violation> {
    let mut out = Vec::new();
    for rule in rules {
        apply_rule(task, roots, rule, &mut out);
    }
    out
}

fn apply_rule(task: &Task, roots: &[HeadingNode], rule: &CompiledRule, out: &mut Vec<Violation>) {
    if !rule_applies(rule, task) {
        return;
    }
    let matched = find_path_matches(roots, &rule.path);
    match rule.action {
        HeaderAction::Require => {
            if matched.is_empty() {
                out.push(require_violation(task, rule));
            }
        }
        HeaderAction::Forbid => {
            for (text, line) in matched {
                out.push(forbid_violation(task, rule, &text, line));
            }
        }
    }
}

fn rule_applies(rule: &CompiledRule, task: &Task) -> bool {
    let Some(filter) = &rule.when else {
        return true;
    };
    filter.matches(task).unwrap_or(false)
}

fn require_violation(task: &Task, rule: &CompiledRule) -> Violation {
    let message = rule
        .message
        .clone()
        .unwrap_or_else(|| format!("required heading not found: {}", describe_path(&rule.path)));
    Violation {
        task_id: task.id.clone(),
        field: "body.headers".to_owned(),
        message,
        location: None,
    }
}

fn forbid_violation(task: &Task, rule: &CompiledRule, text: &str, line: usize) -> Violation {
    let message = rule
        .message
        .clone()
        .unwrap_or_else(|| format!("forbidden heading '{text}'"));
    Violation {
        task_id: task.id.clone(),
        field: "body.headers".to_owned(),
        message,
        location: Some(Location {
            file: task.file.clone(),
            line,
        }),
    }
}

fn find_path_matches(roots: &[HeadingNode], path: &[CompiledSegment]) -> Vec<(String, usize)> {
    let Some((first, rest)) = path.split_first() else {
        return vec![];
    };
    match_in(roots, first, rest)
}

fn match_in(
    scope: &[HeadingNode],
    seg: &CompiledSegment,
    rest: &[CompiledSegment],
) -> Vec<(String, usize)> {
    let mut results = Vec::new();
    let search: Vec<&HeadingNode> = if is_recursive(seg) {
        all_descendants(scope)
    } else {
        scope.iter().collect()
    };
    for entry in search {
        if seg_matches(seg, &entry.heading.text) {
            collect_match(entry, rest, &mut results);
        }
    }
    results
}

fn collect_match(entry: &HeadingNode, rest: &[CompiledSegment], out: &mut Vec<(String, usize)>) {
    if rest.is_empty() {
        out.push((entry.heading.text.clone(), entry.heading.line));
    } else {
        out.extend(find_path_matches(&entry.children, rest));
    }
}

fn all_descendants(scope: &[HeadingNode]) -> Vec<&HeadingNode> {
    let mut found = Vec::new();
    for entry in scope {
        found.push(entry);
        found.extend(all_descendants(&entry.children));
    }
    found
}

fn seg_matches(seg: &CompiledSegment, text: &str) -> bool {
    match seg {
        CompiledSegment::Exact(expected) => expected == text,
        CompiledSegment::Pattern { re, .. } => re.is_match(text),
    }
}

fn is_recursive(seg: &CompiledSegment) -> bool {
    match seg {
        CompiledSegment::Exact(_) => false,
        CompiledSegment::Pattern { recursive, .. } => *recursive,
    }
}

fn describe_path(path: &[CompiledSegment]) -> String {
    path.iter()
        .map(describe_seg)
        .collect::<Vec<_>>()
        .join(" > ")
}

fn describe_seg(seg: &CompiledSegment) -> String {
    match seg {
        CompiledSegment::Exact(text) => format!("'{text}'"),
        CompiledSegment::Pattern {
            re,
            recursive: true,
        } => format!("/{}/..", re.as_str()),
        CompiledSegment::Pattern {
            re,
            recursive: false,
        } => format!("/{}/", re.as_str()),
    }
}

#[cfg(test)]
#[path = "gate_tests.rs"]
mod tests;
