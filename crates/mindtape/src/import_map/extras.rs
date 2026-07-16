//! `extras:` (`x-<ns>` slots) passthrough: validated for the `x-` discipline
//! only — a jq author's structural drops are their own concern, not this
//! layer's (see the module doc on `crate::import_map`).

use std::collections::BTreeMap;

use serde_json::Value;
use typst_harvest::HVal;

use crate::canonical::json_to_hval;

use super::report::ReportEntry;

/// Convert a canonical record's `extras` into `Task::extras`: every `x-<ns>`
/// key round-trips via [`json_to_hval`]; a key missing the `x-` prefix
/// violates the discipline the prelude enforces structurally, so it is
/// dropped and reported rather than written to a field the prelude would
/// reject anyway. Returns the converted map plus the keys carried through
/// unchanged (for [`super::ImportReport::carried_extras`]).
pub(crate) fn map_extras(
    canonical: &BTreeMap<String, Value>,
    out: &mut Vec<ReportEntry>,
) -> (BTreeMap<String, HVal>, Vec<String>) {
    let mut extras = BTreeMap::new();
    let mut carried = Vec::new();
    for (key, value) in canonical {
        if !key.starts_with("x-") {
            out.push(ReportEntry::violated(
                "extras",
                format!("extras key `{key}` does not follow the `x-<ns>` discipline; dropped"),
            ));
            continue;
        }
        extras.insert(key.clone(), json_to_hval(value));
        carried.push(key.clone());
    }
    (extras, carried)
}
