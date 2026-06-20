//! Plane-agnostic structural rule engine.
//!
//! Matches a generic node tree against require/forbid path rules and emits
//! diagnostics. Consumer-blind: no Typst, no task model, no domain concepts.
//!
//! # Example
//!
//! ```
//! use artifact_gate::{Node, Matcher, Rule, Action, run_rules};
//!
//! let roots = vec![Node {
//!     kind: "heading".to_owned(),
//!     text: "Summary".to_owned(),
//!     line: 1,
//!     children: vec![],
//! }];
//! let rule = Rule::new(vec![Matcher::exact("Summary")], Action::Require)
//!     .expect("non-empty path");
//! assert!(run_rules(&roots, &[rule]).is_empty());
//! ```

use thiserror::Error;

/// A generic structure-tree node.
///
/// `kind` is carried for future multi-kind filtering; v1 matching is text-based.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Node kind label (e.g. `"heading"`).
    pub kind: String,
    /// Displayable text content of this node.
    pub text: String,
    /// One-based source line of this node.
    pub line: usize,
    /// Directly nested child nodes.
    pub children: Vec<Node>,
}

/// A single segment matcher in a path rule.
#[derive(Debug, Clone)]
pub enum Matcher {
    /// Exact, case-sensitive, direct-child text match.
    Exact(String),
    /// Regex text pattern; `recursive = true` searches all descendants.
    Pattern {
        /// Compiled regex (boxed to keep enum variant sizes equal).
        re: Box<regex::Regex>,
        /// If `true`, the match searches all descendants, not just direct children.
        recursive: bool,
    },
}

impl PartialEq for Matcher {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Exact(a), Self::Exact(b)) => a == b,
            (
                Self::Pattern {
                    re: ra,
                    recursive: rca,
                },
                Self::Pattern {
                    re: rb,
                    recursive: rcb,
                },
            ) => ra.as_str() == rb.as_str() && rca == rcb,
            _ => false,
        }
    }
}

impl Eq for Matcher {}

impl Matcher {
    /// Construct an exact-text direct-child matcher.
    #[must_use]
    pub fn exact(text: impl Into<String>) -> Self {
        Self::Exact(text.into())
    }

    /// Construct a regex pattern matcher, compiling the regex.
    ///
    /// # Errors
    ///
    /// Returns [`RuleError::Regex`] if the pattern is not a valid regex.
    pub fn pattern(pattern: &str, recursive: bool) -> Result<Self, RuleError> {
        let re = regex::Regex::new(pattern).map_err(|err| RuleError::Regex {
            pattern: pattern.to_owned(),
            error: err.to_string(),
        })?;
        Ok(Self::Pattern {
            re: Box::new(re),
            recursive,
        })
    }
}

/// Whether matched nodes must exist or must not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// The path must match at least one node.
    Require,
    /// The path must match no nodes.
    Forbid,
}

/// Diagnostic severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Severity {
    /// Blocks acceptance (default).
    #[default]
    Error,
    /// Advisory only.
    Warning,
}

/// A compiled, ready-to-run structural rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// Optional identifier for correlating diagnostics back to the rule.
    pub id: Option<String>,
    /// Path of matchers through the tree.
    pub path: Vec<Matcher>,
    /// Whether matching nodes must exist or must not.
    pub action: Action,
    /// Diagnostic severity emitted when the rule triggers.
    pub severity: Severity,
    /// Override for the default diagnostic message.
    pub message: Option<String>,
}

impl Rule {
    /// Build a rule from a path and action.
    ///
    /// # Errors
    ///
    /// Returns [`RuleError::EmptyPath`] if `path` is empty.
    pub fn new(path: Vec<Matcher>, action: Action) -> Result<Self, RuleError> {
        if path.is_empty() {
            return Err(RuleError::EmptyPath);
        }
        Ok(Self {
            id: None,
            path,
            action,
            severity: Severity::default(),
            message: None,
        })
    }

    /// Set a human-readable message override.
    #[must_use]
    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Set a rule identifier.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the diagnostic severity.
    #[must_use]
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }
}

/// A diagnostic emitted by [`run_rules`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The rule id that triggered this diagnostic, if the rule had one.
    pub rule_id: Option<String>,
    /// Severity of the diagnostic.
    pub severity: Severity,
    /// Source line of the offending node, if applicable.
    pub line: Option<usize>,
    /// Human-readable message.
    pub message: String,
}

/// Errors from constructing rules or matchers.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RuleError {
    /// A rule carries an empty path.
    #[error("rule path must not be empty")]
    EmptyPath,
    /// A regex pattern failed to compile.
    #[error("invalid regex `{pattern}`: {error}")]
    Regex {
        /// The offending pattern string.
        pattern: String,
        /// Compiler error message.
        error: String,
    },
}

/// Evaluate `rules` against `roots`, returning all triggered diagnostics.
///
/// For `Require` rules: emits one diagnostic (line = `None`) when no node
/// matches the full path. For `Forbid` rules: emits one diagnostic per matched
/// node, with the node's line number.
#[must_use]
pub fn run_rules(roots: &[Node], rules: &[Rule]) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for rule in rules {
        apply_rule(roots, rule, &mut out);
    }
    out
}

fn apply_rule(roots: &[Node], rule: &Rule, out: &mut Vec<Diagnostic>) {
    let matched = find_path_matches(roots, &rule.path);
    match rule.action {
        Action::Require => {
            if matched.is_empty() {
                out.push(Diagnostic {
                    rule_id: rule.id.clone(),
                    severity: rule.severity,
                    line: None,
                    message: rule.message.clone().unwrap_or_else(|| {
                        format!("required path not found: {}", describe_path(&rule.path))
                    }),
                });
            }
        }
        Action::Forbid => {
            for (text, line) in matched {
                out.push(Diagnostic {
                    rule_id: rule.id.clone(),
                    severity: rule.severity,
                    line: Some(line),
                    message: rule
                        .message
                        .clone()
                        .unwrap_or_else(|| format!("forbidden: {text}")),
                });
            }
        }
    }
}

fn find_path_matches(roots: &[Node], path: &[Matcher]) -> Vec<(String, usize)> {
    let Some((first, rest)) = path.split_first() else {
        return vec![];
    };
    match_in(roots, first, rest)
}

fn match_in(scope: &[Node], seg: &Matcher, rest: &[Matcher]) -> Vec<(String, usize)> {
    let mut results = Vec::new();
    if is_recursive(seg) {
        for entry in all_descendants(scope) {
            if seg_matches(seg, &entry.text) {
                collect_match(entry, rest, &mut results);
            }
        }
    } else {
        for entry in scope {
            if seg_matches(seg, &entry.text) {
                collect_match(entry, rest, &mut results);
            }
        }
    }
    results
}

fn collect_match(entry: &Node, rest: &[Matcher], out: &mut Vec<(String, usize)>) {
    if rest.is_empty() {
        out.push((entry.text.clone(), entry.line));
    } else {
        out.extend(find_path_matches(&entry.children, rest));
    }
}

fn all_descendants(scope: &[Node]) -> Vec<&Node> {
    let mut found = Vec::new();
    for entry in scope {
        found.push(entry);
        found.extend(all_descendants(&entry.children));
    }
    found
}

fn seg_matches(seg: &Matcher, text: &str) -> bool {
    match seg {
        Matcher::Exact(expected) => expected == text,
        Matcher::Pattern { re, .. } => re.is_match(text),
    }
}

fn is_recursive(seg: &Matcher) -> bool {
    match seg {
        Matcher::Exact(_) => false,
        Matcher::Pattern { recursive, .. } => *recursive,
    }
}

fn describe_path(path: &[Matcher]) -> String {
    path.iter()
        .map(describe_seg)
        .collect::<Vec<_>>()
        .join(" > ")
}

fn describe_seg(seg: &Matcher) -> String {
    match seg {
        Matcher::Exact(text) => format!("'{text}'"),
        Matcher::Pattern {
            re,
            recursive: true,
        } => format!("/{}/..", re.as_str()),
        Matcher::Pattern {
            re,
            recursive: false,
        } => format!("/{}/", re.as_str()),
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
