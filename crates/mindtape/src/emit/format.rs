//! Typst literal rendering shared by [`super::emit_task`] and
//! [`super::links`]: string escaping, the array-literal singleton-comma rule,
//! and the `HVal` → Typst literal projection used for `x-<ns>` extras.

use std::collections::BTreeMap;

use typst_harvest::HVal;

/// Quote and escape `text` as a Typst string literal (`"` and `\` only —
/// content-block markup escaping lives in [`super::links`], where a link note
/// rides in a `[...]` block instead of a string literal).
#[must_use]
pub(super) fn typ_str(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

/// A whole-number float still needs a decimal point (`3.0`, not `3`) so Typst
/// evaluates it as a float rather than an int — the two round-trip to
/// different [`HVal`] variants ([`HVal::Float`] vs [`HVal::Int`]), so
/// collapsing the distinction here would break the round-trip property.
fn typ_float(num: f64) -> String {
    if num.is_finite() && num.fract() == 0.0 {
        format!("{num:.1}")
    } else {
        num.to_string()
    }
}

/// A Typst array literal built from already-rendered element strings: `()`
/// empty, `(x,)` one element (bare `(x)` parses as a grouped expression, not
/// a one-element array — Typst's own ambiguity), `(x, y, …)` otherwise.
#[must_use]
pub(super) fn typ_array(elems: &[String]) -> String {
    match elems {
        [] => "()".to_owned(),
        [one] => format!("({one},)"),
        many => format!("({})", many.join(", ")),
    }
}

/// Project one harvested `x-<ns>` value back to a Typst literal: scalars
/// direct, [`HVal::Array`]/[`HVal::Dict`] recursively. A dict key is always
/// quoted (`("k": v, …)`, mirroring `lib/prelude.typ`'s own `"in-review":
/// (...)` entry) so an arbitrary extras key never needs identifier
/// validation.
#[must_use]
pub(super) fn hval_to_typ(value: &HVal) -> String {
    match value {
        HVal::Null => "none".to_owned(),
        HVal::Bool(flag) => flag.to_string(),
        HVal::Int(num) => num.to_string(),
        HVal::Float(num) => typ_float(*num),
        HVal::Str(text) | HVal::Date(text) => typ_str(text),
        HVal::Array(items) => typ_array(&items.iter().map(hval_to_typ).collect::<Vec<_>>()),
        HVal::Dict(map) => typ_dict(map),
    }
}

fn typ_dict(map: &BTreeMap<String, HVal>) -> String {
    if map.is_empty() {
        return "(:)".to_owned();
    }
    let parts: Vec<String> = map
        .iter()
        .map(|(key, value)| format!("{}: {}", typ_str(key), hval_to_typ(value)))
        .collect();
    format!("({})", parts.join(", "))
}
