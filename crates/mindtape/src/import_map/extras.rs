//! `extras:` (`x-<ns>` slots) passthrough: validated for the `x-` discipline
//! only — a jq author's structural drops are their own concern, not this
//! layer's (see the module doc on `crate::import_map`).

use std::collections::BTreeMap;

use serde_json::Value;
use typst_harvest::HVal;

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

/// Convert one JSON value into a harvested [`HVal`] — the inverse of
/// `canonical.rs::hval_to_json`. A JSON number becomes [`HVal::Int`] when it
/// fits an `i64` (matching how [`crate::canonical::hval_to_json`] renders an
/// `HVal::Int`), else [`HVal::Float`]. A JSON string always becomes
/// [`HVal::Str`] — the `HVal::Date`/`HVal::Str` distinction is already lost
/// on the forward projection (`hval_to_json` maps both to a bare JSON
/// string), so this is the only sound reverse; the emitter renders both
/// variants identically anyway (`emit::format::hval_to_typ`).
#[must_use]
pub fn json_to_hval(value: &Value) -> HVal {
    match value {
        Value::Null => HVal::Null,
        Value::Bool(flag) => HVal::Bool(*flag),
        Value::Number(num) => num
            .as_i64()
            .map_or_else(|| HVal::Float(num.as_f64().unwrap_or_default()), HVal::Int),
        Value::String(text) => HVal::Str(text.clone()),
        Value::Array(items) => HVal::Array(items.iter().map(json_to_hval).collect()),
        Value::Object(map) => HVal::Dict(
            map.iter()
                .map(|(key, val)| (key.clone(), json_to_hval(val)))
                .collect(),
        ),
    }
}
