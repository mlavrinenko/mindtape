//! Rule evaluation: path traversal and per-[`Check`](crate::Check) checking.

use crate::{Action, Check, Diagnostic, Matcher, Node, Rule};

/// Evaluate one rule against `roots`, appending any diagnostics.
pub(crate) fn apply_rule(roots: &[Node], rule: &Rule, out: &mut Vec<Diagnostic>) {
    match &rule.check {
        Check::Path { path, action } => apply_path(roots, path, action, rule, out),
        Check::Order { first, second } => apply_order(roots, first, second, rule, out),
        Check::Allow { scope, allowed } => apply_allow(roots, scope.as_deref(), allowed, rule, out),
    }
}

fn apply_path(
    roots: &[Node],
    path: &[Matcher],
    action: &Action,
    rule: &Rule,
    out: &mut Vec<Diagnostic>,
) {
    let mut matched = matches(roots, path);
    match action {
        Action::Require => {
            if matched.is_empty() {
                out.push(missing(rule, path));
            }
        }
        Action::Forbid => {
            for node in matched {
                out.push(at_node(rule, node, || format!("forbidden: {}", node.text)));
            }
        }
        Action::Unique => {
            if matched.is_empty() {
                out.push(missing(rule, path));
            } else {
                matched.sort_by_key(|n| n.line);
                for node in matched.into_iter().skip(1) {
                    out.push(at_node(rule, node, || format!("duplicate: {}", node.text)));
                }
            }
        }
    }
}

fn apply_order(
    roots: &[Node],
    first: &[Matcher],
    second: &[Matcher],
    rule: &Rule,
    out: &mut Vec<Diagnostic>,
) {
    let firsts = matches(roots, first);
    let seconds = matches(roots, second);
    let (Some(max_first), Some(min_second)) = (
        firsts.iter().map(|n| n.line).max(),
        seconds.iter().map(|n| n.line).min(),
    ) else {
        return; // order-only: a missing side is not a violation
    };
    if max_first >= min_second {
        out.push(Diagnostic {
            rule_id: rule.id.clone(),
            severity: rule.severity,
            line: Some(min_second),
            message: rule.message.clone().unwrap_or_else(|| {
                format!(
                    "out of order: {} must precede {}",
                    describe_path(first),
                    describe_path(second)
                )
            }),
        });
    }
}

fn apply_allow(
    roots: &[Node],
    scope: Option<&[Matcher]>,
    allowed: &[Matcher],
    rule: &Rule,
    out: &mut Vec<Diagnostic>,
) {
    let scopes: Vec<&[Node]> = match scope {
        None => vec![roots],
        Some(path) => matches(roots, path)
            .into_iter()
            .map(|n| n.children.as_slice())
            .collect(),
    };
    for children in scopes {
        for child in children {
            if !allowed.iter().any(|m| seg_matches(m, &child.text)) {
                out.push(at_node(rule, child, || {
                    format!("unexpected: {}", child.text)
                }));
            }
        }
    }
}

fn missing(rule: &Rule, path: &[Matcher]) -> Diagnostic {
    Diagnostic {
        rule_id: rule.id.clone(),
        severity: rule.severity,
        line: None,
        message: rule
            .message
            .clone()
            .unwrap_or_else(|| format!("required path not found: {}", describe_path(path))),
    }
}

fn at_node(rule: &Rule, node: &Node, default: impl FnOnce() -> String) -> Diagnostic {
    Diagnostic {
        rule_id: rule.id.clone(),
        severity: rule.severity,
        line: Some(node.line),
        message: rule.message.clone().unwrap_or_else(default),
    }
}

/// Nodes matching the full `path` from `roots`.
fn matches<'a>(roots: &'a [Node], path: &[Matcher]) -> Vec<&'a Node> {
    let Some((seg, rest)) = path.split_first() else {
        return vec![];
    };
    let candidates: Vec<&Node> = if is_recursive(seg) {
        all_descendants(roots)
    } else {
        roots.iter().collect()
    };
    let mut results = Vec::new();
    for entry in candidates {
        if seg_matches(seg, &entry.text) {
            if rest.is_empty() {
                results.push(entry);
            } else {
                results.extend(matches(&entry.children, rest));
            }
        }
    }
    results
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
