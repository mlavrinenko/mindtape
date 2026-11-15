//! Non-slot writable fields for `mt add`: `--set due=…` on a template with no
//! `due` slot inserts the field rather than refusing it.
//!
//! `mt add` fills a slot by rewriting the `slot(…)` call it occupies. A field
//! with no slot has no call to rewrite, and an append needs the list the
//! template already holds. So `mt add` evaluates the draft task first (see
//! [`super::draft`]) and applies the ordered operations to it through the
//! library writer. `mt add` then upserts each touched field into
//! `task.with(…)` as its rendered literal.

use mindtape::model::writable::{self, FieldOp, WritableField};
use mindtape::{SlotSpec, Task};

use super::values::Collected;
use crate::diag::Result;
use crate::task_edit::{task_with, upsert_named_arg};

/// Every writable field the template does not expose as a slot, with its
/// operations in caller order.
pub(super) fn touched(
    collected: &Collected,
    specs: &[SlotSpec],
) -> Vec<(&'static WritableField, Vec<FieldOp>)> {
    writable::WRITABLE
        .iter()
        .filter_map(|field| {
            if specs.iter().any(|spec| spec.name == field.name) {
                return None;
            }
            let ops: Vec<FieldOp> = collected
                .ops
                .iter()
                .filter(|(name, _)| name == field.name)
                .map(|(_, op)| op.clone())
                .collect();
            (!ops.is_empty()).then_some((field, ops))
        })
        .collect()
}

/// The field names in `fields`, for the final upsert and the report.
pub(super) fn names(fields: &[(&'static WritableField, Vec<FieldOp>)]) -> Vec<&'static str> {
    fields.iter().map(|(field, _)| field.name).collect()
}

/// Write every touched field's current value into `source`.
///
/// # Errors
/// A source with no `task.with(…)`, or a rewrite that fails.
pub(super) fn upsert(
    source: &str,
    fields: &[&'static WritableField],
    task: &Task,
) -> Result<String> {
    if fields.is_empty() {
        return Ok(source.to_owned());
    }
    let call = task_with(source)?;
    let mut edits = Vec::new();
    for field in fields {
        let Some(value) = writable::current_literal(field, task) else {
            continue;
        };
        edits.push(upsert_named_arg(source, &call, field.name, &value)?);
    }
    typst_edit::apply(source, edits)
        .map_err(|err| crate::diag::Diag::failed("can't write the field values").cause(&err))
}

#[cfg(test)]
#[path = "field_values_tests.rs"]
mod tests;
