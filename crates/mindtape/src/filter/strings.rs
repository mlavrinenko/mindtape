//! String-valued arguments in the filter language: `word()`, `matches()` and
//! `in_dict()`.
//!
//! Every other function in [`crate::filter`] takes a bare identifier or a
//! literal and answers a boolean, so the walk needed one return type. These
//! three are the seam where an argument may itself be a computed string —
//! `in_dict("verbs", word(title, 1))` — which is what [`string_value`] adds: a
//! second resolution, over the same [`Node`]s, that yields text instead of a
//! verdict.
//!
//! Nothing goes missing quietly. A value that is simply not there — a field the
//! task does not carry, a word past the end of the title — resolves to `None`
//! and reads as a non-match, the same missing-field rule comparisons follow.
//! Everything else is a named [`FilterError`]: an unknown dictionary, an
//! invalid regex, a numeric field where a string is required, a wrong argument
//! count. They surface where the expression is evaluated, exactly as an unknown
//! field name does — and config load dry-runs every saved expression against a
//! probe task, so a view, a limit or a `required` rule reports its own bad
//! filter by name at load rather than quietly matching nothing.

use evalexpr::{DefaultNumericTypes, Node, Operator};
use regex::Regex;

use crate::config::Config;
use crate::filter::expr::{call_args, describe, extract_literal, restore_name, unwrap_root};
use crate::filter::{FilterError, field_value};
use crate::model::Task;
use crate::model::fields::FieldValue;

/// Resolve a node standing where a string is expected: a literal, a field
/// identifier, or a nested string-valued call.
///
/// `Ok(None)` is "there is no such value" — a field the task does not carry, a
/// word index past the end — which every caller reads as a non-match.
///
/// # Errors
/// [`FilterError::UnknownField`] for a name the registry does not know, and
/// [`FilterError::Invalid`] for a numeric field, a boolean-valued call, or an
/// operator that produces no string at all.
pub(crate) fn string_value(
    node: &Node<DefaultNumericTypes>,
    task: &Task,
    config: &Config,
) -> Result<Option<String>, FilterError> {
    let node = unwrap_root(node);
    match node.operator() {
        Operator::Const { .. } | Operator::Neg => extract_literal(node)
            .map(Some)
            .map_err(FilterError::Invalid),
        Operator::VariableIdentifierRead { identifier } => {
            string_field(task, &restore_name(identifier), config)
        }
        Operator::FunctionIdentifier { identifier } => {
            string_call(&restore_name(identifier), node.children(), task, config)
        }
        _ => Err(FilterError::Invalid(format!(
            "expected a string, a field or word(...), got {}",
            describe(node)
        ))),
    }
}

/// A field read in string position. A numeric field is an error rather than a
/// formatted number: `word(priority, 1)` is a rule that cannot mean anything,
/// and reading it as `"8"` would make it silently almost-work. An `x-<ns>`
/// list holds no single string and reads as no value, as it equals nothing
/// under `==`: one slot may hold a string on one task and a list on the next.
fn string_field(task: &Task, name: &str, config: &Config) -> Result<Option<String>, FilterError> {
    match field_value(task, name, config)? {
        FieldValue::Str(text) => Ok(Some(text)),
        FieldValue::Missing | FieldValue::List(_) => Ok(None),
        FieldValue::Num(_) => Err(FilterError::Invalid(format!(
            "'{name}' is a number; word(), matches() and in_dict() read text fields"
        ))),
    }
}

/// The calls that yield a string. Only `word()` does today; every other
/// function in the language answers a boolean, so naming one here is a type
/// error rather than an unknown name.
fn string_call(
    name: &str,
    children: &[Node<DefaultNumericTypes>],
    task: &Task,
    config: &Config,
) -> Result<Option<String>, FilterError> {
    match name {
        "word" => word_call(children, task, config),
        other => Err(FilterError::call(
            format!("'{other}(...)' is a condition, not text; only word(...) gives text"),
            "in_dict(\"verbs\", word(title, 1))",
        )),
    }
}

/// `word(field, n)` — the n-th whitespace-separated token of a string value,
/// counting from 1, trimmed of every non-alphanumeric character at either end
/// (`"Fix:"` is the word `Fix`).
///
/// It takes no position on Typst markup: a title opening with a code span
/// yields the token inside the ticks, because the token is all a whitespace
/// split can see. Exempting such a title is the rule author's business, which
/// is what [`matches_call`] is for.
fn word_call(
    children: &[Node<DefaultNumericTypes>],
    task: &Task,
    config: &Config,
) -> Result<Option<String>, FilterError> {
    let args = call_args(children);
    let [value, index] = args.as_slice() else {
        return Err(arity("word", "word(title, 1)"));
    };
    let position = word_index(index)?;
    let Some(text) = string_value(value, task, config)? else {
        return Ok(None);
    };
    Ok(nth_word(&text, position))
}

/// The n-th word of `text`, or `None` past the end — or when the token carries
/// no alphanumeric character at all, which trims to nothing and reads the same
/// as no word there.
fn nth_word(text: &str, position: usize) -> Option<String> {
    let word = text
        .split_whitespace()
        .nth(position - 1)?
        .trim_matches(|c: char| !c.is_alphanumeric());
    (!word.is_empty()).then(|| word.to_owned())
}

/// The 1-based position argument. Zero and a negative both fail with the same
/// message, because both are the same misreading of where counting starts.
fn word_index(node: &Node<DefaultNumericTypes>) -> Result<usize, FilterError> {
    let literal = extract_literal(node).map_err(FilterError::Invalid)?;
    match literal.parse::<usize>() {
        Ok(position) if position >= 1 => Ok(position),
        _ => Err(FilterError::call(
            format!("word() counts from 1, not '{literal}'"),
            "word(title, 1)",
        )),
    }
}

/// `matches(field, "regex")` — the escape hatch a closed vocabulary needs. The
/// pattern is a Rust `regex`, unanchored, and must be a literal: it is part of
/// the rule, not part of the data.
///
/// # Errors
/// [`FilterError::Invalid`] on a wrong argument count, a non-literal pattern,
/// or a pattern `regex` refuses; whatever [`string_value`] raises for the first
/// argument.
pub(crate) fn matches_call(
    children: &[Node<DefaultNumericTypes>],
    task: &Task,
    config: &Config,
) -> Result<bool, FilterError> {
    let args = call_args(children);
    let [value, pattern] = args.as_slice() else {
        return Err(arity("matches", "matches(title, \"^Add \")"));
    };
    let pattern = extract_literal(pattern).map_err(|_| {
        FilterError::call(
            "matches() takes its pattern second, as a quoted string",
            "matches(title, \"^Add \")",
        )
    })?;
    let regex = Regex::new(&pattern).map_err(|err| {
        FilterError::Invalid(format!(
            "matches() was given a bad regex '{pattern}': {err}"
        ))
    })?;
    Ok(string_value(value, task, config)?.is_some_and(|text| regex.is_match(&text)))
}

/// `in_dict("name", <string expr>)` — membership in a configured or built-in
/// dictionary, under that dictionary's own `case`.
///
/// The name is validated here, where it is written, rather than at lookup:
/// [`crate::config::Dicts::contains`] reads an unknown name as a miss, which
/// would turn a typo into a rule that silently never fires.
///
/// # Errors
/// [`FilterError::UnknownDict`] for a name no compiled dictionary carries, and
/// [`FilterError::Invalid`] on a wrong argument count or a non-literal name;
/// whatever [`string_value`] raises for the second argument.
pub(crate) fn in_dict_call(
    children: &[Node<DefaultNumericTypes>],
    task: &Task,
    config: &Config,
) -> Result<bool, FilterError> {
    let args = call_args(children);
    let [name, value] = args.as_slice() else {
        return Err(arity("in_dict", "in_dict(\"verbs\", word(title, 1))"));
    };
    let name = extract_literal(name).map_err(|_| {
        FilterError::call(
            "in_dict() takes the dictionary name first, as a quoted string",
            "in_dict(\"verbs\", word(title, 1))",
        )
    })?;
    let dicts = config.compiled_dicts();
    let dict = dicts.get(&name).ok_or_else(|| {
        let known: Vec<String> = dicts.names().map(ToOwned::to_owned).collect();
        FilterError::UnknownDict {
            hint: crate::suggest::hint("dict", &name, &known),
            name: name.clone(),
        }
    })?;
    Ok(string_value(value, task, config)?.is_some_and(|text| dict.contains(&text)))
}

/// `word(...)` reached boolean position. Named rather than left to the unknown
/// function arm, because the name IS known — it is in the wrong place.
pub(crate) fn word_is_not_boolean() -> FilterError {
    FilterError::call(
        "word() gives text, not a condition",
        "in_dict(\"verbs\", word(title, 1))",
    )
}

fn arity(name: &str, example: &'static str) -> FilterError {
    FilterError::call(format!("{name}() takes two arguments"), example)
}

#[cfg(test)]
#[path = "strings_tests.rs"]
mod tests;
