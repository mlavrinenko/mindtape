//! `starts_with("text", field)`: the one-field prefix check. Split out of
//! `predicates.rs` (file-size gate), and built on that module's reader and case
//! fold, so the two text predicates cannot drift apart on which fields they
//! read or how they compare case.
//!
//! The needle comes first and the field second, and the field is required:
//! unlike [`contains()`](crate::filter::predicates::contains_call), a prefix
//! never falls back to a haystack the caller did not name.

use evalexpr::{DefaultNumericTypes, Node};

use crate::filter::expr::{extract_identifier, extract_literal};
use crate::filter::predicates::{contains_value, fold_case};
use crate::filter::strings::call_pair;
use crate::filter::{Ctx, FilterError};
use crate::model::fields::{self as fields, FieldValue};

/// The example every `starts_with()` error carries.
const EXAMPLE: &str = "starts_with(\"fix\", title)";

/// `starts_with("text", field)`: whether the named text field opens with the
/// text, ignoring case. The field reads through
/// [`contains_value`], so `body`, `note` and `tags` read as `contains()` reads
/// them, and the field vocabulary is the same.
///
/// # Errors
/// [`FilterError::Call`] naming `starts_with` and showing [`EXAMPLE`] for a
/// wrong argument count, a non-literal needle, a field that is not a bare
/// identifier, a name outside `contains()`'s vocabulary, or a field whose
/// value is not text. A missing value is no match, as it is under `==`.
pub(crate) fn starts_with_call(
    children: &[Node<DefaultNumericTypes>],
    ctx: &Ctx,
) -> Result<bool, FilterError> {
    let [needle, field] = call_pair(children, "starts_with", EXAMPLE)?;
    let needle = extract_literal(needle).map_err(|err| {
        FilterError::call(
            format!("starts_with() takes the text first: {err}"),
            EXAMPLE,
        )
    })?;
    let name = extract_identifier(field).map_err(|err| {
        FilterError::call(
            format!("starts_with() takes one field second: {err}"),
            EXAMPLE,
        )
    })?;
    if !fields::is_contains_field(&name) {
        return Err(FilterError::call(
            format!("starts_with() reads a field contains() searches, not '{name}'"),
            EXAMPLE,
        ));
    }
    match contains_value(ctx.task, &name, ctx.config)? {
        FieldValue::Str(text) => Ok(fold_case(&text).starts_with(&fold_case(&needle))),
        FieldValue::Missing => Ok(false),
        FieldValue::Num(_) | FieldValue::List(_) | FieldValue::Record => Err(FilterError::call(
            format!("starts_with() reads text, and '{name}' is not text"),
            EXAMPLE,
        )),
    }
}
