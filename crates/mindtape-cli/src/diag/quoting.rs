//! The quoted form of a filter whose bare word was meant as text:
//! `status == wip` reads `wip` as a field, and `status == "wip"` is what the
//! user meant. Built from the user's own expression, so it names no status.

/// Whether `c` can sit inside a field name, as in `estimate.src-files`.
fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || "_-.".contains(c)
}

/// The comparison operators, the two-character ones first so `<=` never
/// reads as `<`.
const COMPARISONS: [&str; 6] = ["==", "!=", "<=", ">=", "<", ">"];

/// `expr` with each bare `name` that a comparison holds against another bare
/// name quoted as text, or `None` when no comparison holds it so. A `name`
/// inside a string, inside a longer name or compared with a literal stays as
/// it is.
pub(super) fn quoted_form(expr: &str, name: &str) -> Option<String> {
    let spans: Vec<(usize, usize)> = bare_spans(expr, name)
        .into_iter()
        .filter(|&(start, end)| left_compared(&expr[..start]) || right_compared(&expr[end..]))
        .collect();
    if spans.is_empty() {
        return None;
    }
    let mut out = String::new();
    let mut last = 0;
    for (start, end) in spans {
        out.push_str(&expr[last..start]);
        out.push_str(&format!("\"{name}\""));
        last = end;
    }
    out.push_str(&expr[last..]);
    Some(out)
}

/// The byte spans of `name` in `expr` outside any string and not part of a
/// longer name.
fn bare_spans(expr: &str, name: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut prev: Option<char> = None;
    for (at, c) in expr.char_indices() {
        if in_string {
            in_string = escaped || c != '"';
            escaped = !escaped && c == '\\';
        } else if c == '"' {
            in_string = true;
        } else if expr[at..].starts_with(name) && !prev.is_some_and(is_name_char) {
            let end = at + name.len();
            if !expr[end..].chars().next().is_some_and(is_name_char) {
                spans.push((at, end));
            }
        }
        prev = Some(c);
    }
    spans
}

/// Whether `before` ends with a comparison whose left side ends in a name or
/// a call: `status ==`, `now() <`.
fn left_compared(before: &str) -> bool {
    across_comparison(before, true).is_some_and(|c| is_name_char(c) || c == ')')
}

/// Whether `after` starts with a comparison whose right side starts with a
/// name: `== status`.
fn right_compared(after: &str) -> bool {
    across_comparison(after, false).is_some_and(|c| c.is_alphabetic() || c == '_')
}

/// The character across the comparison operator that `side` opens with, or
/// closes with when `from_end`, spaces skipped; `None` when no operator is
/// there.
fn across_comparison(side: &str, from_end: bool) -> Option<char> {
    COMPARISONS.iter().find_map(|op| {
        if from_end {
            let rest = side.trim_end().strip_suffix(op)?;
            rest.trim_end().chars().next_back()
        } else {
            let rest = side.trim_start().strip_prefix(op)?;
            rest.trim_start().chars().next()
        }
    })
}
