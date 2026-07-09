//! View-reference sugar for the filter language.
//!
//! Two lexical/structural helpers that keep the `@name` view sugar out of the
//! evaluator proper:
//!
//! - [`desugar_view_refs`] rewrites `@name` to `in_view("name")` at parse time,
//!   skipping any `@` inside a string literal, so [`crate::filter::Filter`] and
//!   evalexpr only ever see the `in_view(...)` form.
//! - [`collect_view_refs`] walks a parsed tree gathering the view names its
//!   `in_view(...)` calls reference, for load-time resolution and cycle checks.

use evalexpr::{DefaultNumericTypes, Node, Operator};

use crate::filter::extract_literal;

/// Rewrite `@name` to `in_view("name")`, leaving `@` inside string literals
/// untouched. `name` is `[A-Za-z0-9_-]+`; a bare `@` (not followed by a name
/// character) passes through verbatim.
#[must_use]
pub(crate) fn desugar_view_refs(expr: &str) -> String {
    let mut out = String::with_capacity(expr.len() + 16);
    let mut chars = expr.char_indices().peekable();
    let mut in_string = false;
    while let Some((idx, ch)) = chars.next() {
        if in_string {
            out.push(ch);
            if ch == '\\' {
                if let Some((_, escaped)) = chars.next() {
                    out.push(escaped);
                }
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => {
                in_string = true;
                out.push('"');
            }
            '@' => {
                let name_start = idx + ch.len_utf8();
                let mut name_end = name_start;
                while let Some(&(next_idx, next)) = chars.peek() {
                    if next.is_ascii_alphanumeric() || next == '_' || next == '-' {
                        name_end = next_idx + next.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                }
                if name_end > name_start {
                    out.push_str("in_view(\"");
                    out.push_str(&expr[name_start..name_end]);
                    out.push_str("\")");
                } else {
                    out.push('@');
                }
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Collect the view names referenced by `in_view("name")` calls anywhere in
/// `node`, appending to `out` in traversal order (may contain duplicates).
pub(crate) fn collect_view_refs(node: &Node<DefaultNumericTypes>, out: &mut Vec<String>) {
    if let Operator::FunctionIdentifier { identifier } = node.operator()
        && identifier == "in_view"
        && let [arg] = node.children()
        && let Ok(name) = extract_literal(arg)
    {
        out.push(name);
    }
    for child in node.children() {
        collect_view_refs(child, out);
    }
}

#[cfg(test)]
#[path = "filter_views_tests.rs"]
mod tests;
