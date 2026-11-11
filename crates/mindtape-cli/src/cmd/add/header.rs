//! The `task.with(…)` header edits `mt add` owns: config-required slots the
//! template omits, and the status/id an add stamps once.
//!
//! Split out of `cmd/add/mod.rs` for the file-size gate. `mt check`'s template
//! pass shares both functions, so they stay `pub(crate)`.

use mindtape::{Config, RequiredSlot, status_value, today_ymd};
use typst_edit::apply;

use crate::diag::{Diag, Result};
use crate::task_edit::{initial_status, task_with, upsert_named_arg};

/// Splice slot-bearing values for config-required core fields the template
/// omits, so their slots surface in the fill scaffold, `--describe`, and the
/// written task. A field already set by the template (literal or slot) is left
/// untouched, except that a slot which is its whole value inherits the field's
/// config `values` as choices ([`mindtape::inherit_field_choices`]). Runs
/// before discovery, so a spliced value may itself nest slots — a
/// strict-framework priority splices `framework("ice", impact: slot(…), …)`.
///
/// Returns the fields actually spliced (the template omitted them), so the
/// caller can report the values it filled that the template never mentioned.
pub(crate) fn splice_config_slots(
    source: &str,
    config: &Config,
    status: Option<&str>,
) -> Result<(String, Vec<RequiredSlot>)> {
    let source =
        mindtape::inherit_field_choices(source, config).map_err(|err| edit_failed(&err))?;
    let call = task_with(&source)?;
    let mut edits = Vec::new();
    let mut injected = Vec::new();
    for slot in mindtape::required_core_slots(config, status) {
        if call.named(&slot.field).is_none() {
            edits.push(upsert_named_arg(&source, &call, &slot.field, &slot.call)?);
            injected.push(slot);
        }
    }
    let spliced = apply(&source, edits).map_err(|err| edit_failed(&err))?;
    Ok((spliced, injected))
}

/// Inject the add-owned `status` (and `id` when the config generates one) into
/// `task.with(…)`.
pub(crate) fn inject_header(
    source: &str,
    config: &Config,
    status_override: Option<&str>,
) -> Result<String> {
    let call = task_with(source)?;
    let mut edits = Vec::new();

    let status_name = status_override
        .map(ToOwned::to_owned)
        .or_else(|| initial_status(config));
    if let Some(status_name) = status_name {
        let value = status_value(&status_name, today_ymd(), None);
        edits.push(upsert_named_arg(source, &call, "status", &value)?);
    }

    if let Some(id) = config.tasks.id.format.generate() {
        edits.push(upsert_named_arg(source, &call, "id", &format!("\"{id}\""))?);
    }

    apply(source, edits).map_err(|err| edit_failed(&err))
}

/// The template's edits overlapped or ran out of bounds.
pub(super) fn edit_failed(err: &typst_edit::EditError) -> Diag {
    Diag::failed("can't rewrite the template").cause(err)
}
