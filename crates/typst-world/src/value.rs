//! A small, serde-style value tree that decouples consumers from raw Typst types.
//!
//! [`HVal`] mirrors the subset of Typst values that appear in metadata markers:
//! scalars, dates (rendered `YYYY-MM-DD`), arrays, and dictionaries. Consumers
//! read harvested markers as `HVal` and never depend on `typst::foundations`.

use std::collections::BTreeMap;

use typst::foundations::{Content, Datetime, Repr, Value};
use typst_library::model::{Destination, LinkElem, LinkTarget};

/// A harvested value: the Typst-agnostic projection of a `metadata()` payload.
#[derive(Debug, Clone, PartialEq)]
pub enum HVal {
    /// Typst `none`.
    Null,
    /// Boolean.
    Bool(bool),
    /// Integer.
    Int(i64),
    /// Float.
    Float(f64),
    /// String.
    Str(String),
    /// Date rendered as `YYYY-MM-DD`.
    Date(String),
    /// Ordered array.
    Array(Vec<HVal>),
    /// String-keyed dictionary (sorted by key).
    Dict(BTreeMap<String, HVal>),
}

impl HVal {
    /// Borrow the string payload, if this is a [`HVal::Str`] or [`HVal::Date`].
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(text) | Self::Date(text) => Some(text),
            _ => None,
        }
    }

    /// The integer payload, if this is a [`HVal::Int`].
    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(*value),
            _ => None,
        }
    }

    /// The numeric payload as `f64`, accepting both ints and floats.
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Int(value) => Some(*value as f64),
            Self::Float(value) => Some(*value),
            _ => None,
        }
    }

    /// Borrow the array payload, if this is a [`HVal::Array`].
    #[must_use]
    pub fn as_array(&self) -> Option<&[HVal]> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }

    /// Borrow the dictionary payload, if this is a [`HVal::Dict`].
    #[must_use]
    pub fn as_dict(&self) -> Option<&BTreeMap<String, HVal>> {
        match self {
            Self::Dict(map) => Some(map),
            _ => None,
        }
    }

    /// Look up a key in a [`HVal::Dict`].
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&HVal> {
        self.as_dict().and_then(|map| map.get(key))
    }

    /// Whether this is [`HVal::Null`].
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

/// Render a Typst [`Datetime`] as `YYYY-MM-DD`, leaving absent parts as zero.
#[must_use]
pub fn format_date(date: &Datetime) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        date.year().unwrap_or(0),
        date.month().unwrap_or(0),
        date.day().unwrap_or(0),
    )
}

/// Convert a Typst [`Value`] into an [`HVal`]. Unsupported kinds (functions,
/// content, …) collapse to [`HVal::Str`] of their debug-ish repr so harvesting
/// never fails on an exotic payload.
#[must_use]
pub fn convert(value: &Value) -> HVal {
    match value {
        Value::None => HVal::Null,
        Value::Bool(flag) => HVal::Bool(*flag),
        Value::Int(num) => HVal::Int(*num),
        Value::Float(num) => HVal::Float(*num),
        Value::Str(text) => HVal::Str(text.as_str().to_owned()),
        Value::Datetime(date) => HVal::Date(format_date(date)),
        Value::Array(items) => HVal::Array(items.iter().map(convert).collect()),
        Value::Dict(map) => HVal::Dict(
            map.iter()
                .map(|(key, val)| (key.as_str().to_owned(), convert(val)))
                .collect(),
        ),
        Value::Content(content) => content_str(content),
        other => HVal::Str(other.repr().as_str().to_owned()),
    }
}

/// Project a content value to a plain string.
///
/// Two consumer-blind rules, knowing nothing of tasks:
///   - A standard `link("…")` with a URL destination carries the bare target
///     (how authors express cross-references and task↔task edges, since the
///     prelude must not shadow the builtin `link`); harvest its destination.
///   - Any other content (a trailing `[note]` block, a title, …) projects to
///     its concatenated plain text.
///
/// Label / page-position link destinations have no string target, so they fall
/// through to the plain-text rule.
fn content_str(content: &Content) -> HVal {
    if let Some(link) = content.to_packed::<LinkElem>()
        && let LinkTarget::Dest(Destination::Url(url)) = &link.dest
    {
        return HVal::Str(url.as_str().to_owned());
    }
    HVal::Str(content.plain_text().as_str().to_owned())
}

#[cfg(test)]
#[path = "value_tests.rs"]
mod tests;
