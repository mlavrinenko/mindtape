//! Plane-agnostic structural rule engine.
//!
//! Matches a generic node tree against structural rules and emits diagnostics.
//! Consumer-blind: no Typst, no task model, no domain concepts.
//!
//! Three rule shapes ([`Check`]):
//!
//! - [`Check::Path`] — cardinality of a path's matches ([`Action::Require`] /
//!   [`Action::Forbid`] / [`Action::Unique`]).
//! - [`Check::Order`] — `first` matches must all precede `second` matches in
//!   document (line) order.
//! - [`Check::Allow`] — closed-world: under a scope (or the roots), only the
//!   `allowed` matchers are permitted; anything else is flagged.
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
//! let rule = Rule::path(vec![Matcher::exact("Summary")], Action::Require)
//!     .expect("non-empty path");
//! assert!(run_rules(&roots, &[rule]).is_empty());
//! ```

mod eval;

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

/// Cardinality demanded of a [`Check::Path`] rule's matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// The path must match at least one node.
    Require,
    /// The path must match no nodes.
    Forbid,
    /// The path must match exactly one node.
    Unique,
}

/// The structural predicate a [`Rule`] enforces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    /// Cardinality constraint on the matches of a single path.
    Path {
        /// Path of matchers through the tree.
        path: Vec<Matcher>,
        /// Required cardinality of the matches.
        action: Action,
    },
    /// Document-order constraint: every `first` match must precede every
    /// `second` match (by line). Order-only — neither side is required to exist.
    Order {
        /// Path whose matches must come first.
        first: Vec<Matcher>,
        /// Path whose matches must come after.
        second: Vec<Matcher>,
    },
    /// Closed-world constraint. Within `scope`'s matched nodes' direct children
    /// (or the roots when `scope` is `None`), only nodes matching one of
    /// `allowed` are permitted; any other node is flagged.
    Allow {
        /// Optional path selecting the scopes whose children are constrained;
        /// `None` constrains the root nodes.
        scope: Option<Vec<Matcher>>,
        /// Matchers naming the permitted child nodes.
        allowed: Vec<Matcher>,
    },
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

/// A compiled, ready-to-run structural rule: a [`Check`] plus diagnostic metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// Optional identifier for correlating diagnostics back to the rule.
    pub id: Option<String>,
    /// Diagnostic severity emitted when the rule triggers.
    pub severity: Severity,
    /// Override for the default diagnostic message.
    pub message: Option<String>,
    /// The structural predicate this rule enforces.
    pub check: Check,
}

impl Rule {
    /// Build a cardinality rule from a path and action.
    ///
    /// # Errors
    ///
    /// Returns [`RuleError::EmptyPath`] if `path` is empty.
    pub fn path(path: Vec<Matcher>, action: Action) -> Result<Self, RuleError> {
        if path.is_empty() {
            return Err(RuleError::EmptyPath);
        }
        Ok(Self::from_check(Check::Path { path, action }))
    }

    /// Build an ordering rule: `first` matches must precede `second` matches.
    ///
    /// # Errors
    ///
    /// Returns [`RuleError::EmptyPath`] if either path is empty.
    pub fn order(first: Vec<Matcher>, second: Vec<Matcher>) -> Result<Self, RuleError> {
        if first.is_empty() || second.is_empty() {
            return Err(RuleError::EmptyPath);
        }
        Ok(Self::from_check(Check::Order { first, second }))
    }

    /// Build a closed-world allowlist rule.
    ///
    /// `scope` of `None` constrains the root nodes; `Some(path)` constrains the
    /// direct children of every node the path matches.
    ///
    /// # Errors
    ///
    /// Returns [`RuleError::EmptyPath`] if `allowed` is empty, or if `scope` is
    /// `Some` and empty.
    pub fn allow(scope: Option<Vec<Matcher>>, allowed: Vec<Matcher>) -> Result<Self, RuleError> {
        if allowed.is_empty() || scope.as_ref().is_some_and(Vec::is_empty) {
            return Err(RuleError::EmptyPath);
        }
        Ok(Self::from_check(Check::Allow { scope, allowed }))
    }

    fn from_check(check: Check) -> Self {
        Self {
            id: None,
            severity: Severity::default(),
            message: None,
            check,
        }
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
    /// A rule carries an empty matcher path.
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
/// - [`Check::Path`] with [`Action::Require`]: one diagnostic (line `None`) when
///   no node matches. [`Action::Forbid`]: one per matched node, with its line.
///   [`Action::Unique`]: a missing diagnostic when absent, else one per
///   duplicate (the second and later matches), each at its line.
/// - [`Check::Order`]: one diagnostic (at the earliest offending `second` line)
///   when both sides are present and out of order.
/// - [`Check::Allow`]: one diagnostic per disallowed node, at its line.
#[must_use]
pub fn run_rules(roots: &[Node], rules: &[Rule]) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for rule in rules {
        eval::apply_rule(roots, rule, &mut out);
    }
    out
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
