//! A syntax error in plain words. `evalexpr` words its errors in capitalised
//! sentences and Rust type names, so [`plain_parse_error`] rewrites each it
//! can raise, and [`refuse_stray_symbol`] catches the symbols it quietly
//! reads as names.

use evalexpr::EvalexprError;

use crate::filter::FilterError;

/// The symbols `evalexpr` lexes as part of a name, so `title ~ "x"` parses as
/// a call and fails later with a misleading message.
const STRAY: [char; 4] = ['~', '$', '?', '#'];

/// Refuse a [`STRAY`] symbol outside a string literal.
///
/// # Errors
/// [`FilterError::Call`] for `~`, which suggests `matches()`, and
/// [`FilterError::Invalid`] for the others.
pub(crate) fn refuse_stray_symbol(expr: &str) -> Result<(), FilterError> {
    let (mut in_string, mut escaped) = (false, false);
    for ch in expr.chars() {
        if in_string {
            in_string = escaped || ch != '"';
            escaped = !escaped && ch == '\\';
        } else if ch == '"' {
            in_string = true;
        } else if ch == '~' {
            return Err(FilterError::Call {
                problem: "'~' isn't an operator".to_owned(),
                example: "matches(title, \"x\")",
            });
        } else if STRAY.contains(&ch) {
            return Err(FilterError::Invalid(format!("'{ch}' isn't an operator")));
        }
    }
    Ok(())
}

/// The reason `err` gives, in the filter's own words, with no trailing period.
pub(crate) fn plain_parse_error(err: &EvalexprError) -> String {
    match err {
        EvalexprError::UnmatchedLBrace => "a '(' is never closed".to_owned(),
        EvalexprError::UnmatchedRBrace => "a ')' has no '(' before it".to_owned(),
        EvalexprError::UnmatchedDoubleQuote => "a '\"' is never closed".to_owned(),
        EvalexprError::MissingOperatorOutsideOfBrace => {
            "a parenthesis sits where there is nothing for it to group".to_owned()
        }
        EvalexprError::AppendedToLeafNode => {
            "two values sit side by side with no operator between them".to_owned()
        }
        EvalexprError::PrecedenceViolation => {
            "the operators can't be ordered; add parentheses".to_owned()
        }
        EvalexprError::IllegalEscapeSequence(sequence) => {
            format!("'{sequence}' isn't an escape a string takes")
        }
        EvalexprError::UnmatchedPartialToken { .. } => partial_token(&err.to_string()),
        EvalexprError::CustomMessage(message) => message.clone(),
        other => other.to_string().trim_end_matches('.').to_lowercase(),
    }
}

/// `&` and `|` alone, which `evalexpr` reports as a partial token: the fix is
/// the doubled operator.
fn partial_token(shown: &str) -> String {
    let symbol = shown
        .strip_prefix("Found a partial token '")
        .and_then(|rest| rest.chars().next());
    match symbol {
        Some('&') => "'&' isn't an operator; write '&&'".to_owned(),
        Some('|') => "'|' isn't an operator; write '||'".to_owned(),
        _ => "an operator is cut short".to_owned(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::filter::{Filter, FilterError};

    fn reason(expr: &str) -> String {
        match Filter::parse(expr).unwrap_err() {
            FilterError::Parse(reason) => reason,
            other => other.to_string(),
        }
    }

    #[test]
    fn each_syntax_error_is_said_in_plain_words() {
        for (expr, expected) in [
            ("foo(", "a '(' is never closed"),
            ("a == 1)", "a ')' has no '(' before it"),
            ("a == \"x", "a '\"' is never closed"),
            ("a & b", "'&' isn't an operator; write '&&'"),
            ("a | b", "'|' isn't an operator; write '||'"),
            (
                "a == 1 1",
                "two values sit side by side with no operator between them",
            ),
            ("\"a\\q\"", "'\\q' isn't an escape a string takes"),
            (
                "1 (2)",
                "a parenthesis sits where there is nothing for it to group",
            ),
            ("/* x", "unmatched inline comment"),
            ("a &&& b", "'&' isn't an operator; write '&&'"),
        ] {
            assert_eq!(reason(expr), expected, "{expr}");
        }
    }

    #[test]
    fn a_stray_symbol_outside_a_string_is_refused_and_inside_one_is_not() {
        assert!(matches!(
            Filter::parse("title ~ \"x\"").unwrap_err(),
            FilterError::Call { example, .. } if example.starts_with("matches(")
        ));
        assert!(matches!(
            Filter::parse("a $ b").unwrap_err(),
            FilterError::Invalid(reason) if reason == "'$' isn't an operator"
        ));
        assert!(Filter::parse("title == \"a ~ b\"").is_ok());
    }
}
