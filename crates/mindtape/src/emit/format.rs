//! Typst literal rendering shared by [`super::emit_task`] and
//! [`super::links`]: string escaping, the array-literal singleton-comma rule,
//! and the `HVal` → Typst literal projection used for `x-<ns>` extras.
//!
//! `typ_str` and `typ_array` are exported from the crate root: rendering a
//! value as a Typst literal is what a writer outside this module needs too —
//! `mt set` renders one field's value with the same two calls that emit a
//! whole task — and a second copy of the escaping rule, or of the
//! singleton-comma rule below, is a wrong file waiting to be written.
//!
//! All three are read beyond `emit` inside the crate as well:
//! [`crate::rules::typst`] builds a Typst literal for a `[[tasks.rules]]
//! check` function's argument from the same [`HVal`] tree, but renders a
//! [`HVal::Date`] as a real `datetime(...)` call instead of the quoted string
//! [`hval_to_typ`] writes for round-tripping `x-<ns>` extras verbatim — a
//! second dispatcher reusing these three, not a fork of them.

use std::collections::BTreeMap;

use typst_harvest::HVal;

/// Quote and escape `text` as a Typst string literal (`"` and `\` only —
/// content-block markup escaping lives in [`super::links`], where a link note
/// rides in a `[...]` block instead of a string literal).
#[must_use]
pub fn typ_str(text: &str) -> String {
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
pub fn typ_array(elems: &[String]) -> String {
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
pub(crate) fn hval_to_typ(value: &HVal) -> String {
    match value {
        HVal::Null => "none".to_owned(),
        HVal::Bool(flag) => flag.to_string(),
        HVal::Int(num) => num.to_string(),
        HVal::Float(num) => typ_float(*num),
        HVal::Str(text) | HVal::Date(text) => typ_str(text),
        HVal::Array(items) => typ_array(&items.iter().map(hval_to_typ).collect::<Vec<_>>()),
        HVal::Dict(map) => typ_dict_with(map, hval_to_typ),
    }
}

/// A Typst dict literal built from already-rendered element strings, keyed by
/// `map`'s own keys: `(:)` empty, `("k": v, …)` otherwise — every key quoted,
/// mirroring `lib/prelude.typ`'s own `"in-review": (...)` entry, so an
/// arbitrary extras key never needs identifier validation. `render` projects
/// one value to its literal; [`hval_to_typ`] passes itself back in for the
/// recursive case, and [`crate::rules::typst`] passes its own
/// datetime-aware dispatcher instead.
#[must_use]
pub(crate) fn typ_dict_with(
    map: &BTreeMap<String, HVal>,
    render: impl Fn(&HVal) -> String,
) -> String {
    if map.is_empty() {
        return "(:)".to_owned();
    }
    let parts: Vec<String> = map
        .iter()
        .map(|(key, value)| format!("{}: {}", typ_str(key), render(value)))
        .collect();
    format!("({})", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typ_str_escapes_backslash() {
        assert_eq!(typ_str(r"a\b"), r#""a\\b""#);
    }

    #[test]
    fn typ_float_finite_non_whole_uses_to_string() {
        assert_eq!(typ_float(1.23), "1.23");
    }
}
