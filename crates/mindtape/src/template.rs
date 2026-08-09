//! Slot-template engine: discover `slot(...)` calls and fill them by source
//! rewrite.
//!
//! A template is an ordinary task artifact whose authored values are `slot(...)`
//! calls (see the prelude helper). This module finds each call on the parse plane
//! (`typst_edit::find_calls` — no eval, no `World`), reads its literal arguments
//! into a [`SlotSpec`], and [`fill`]s the source: each slot is replaced by a
//! rendered literal so the filed task holds plain values and never calls `slot`.
//!
//! Slot arguments must be literals; this engine never evaluates the template.
//!
//! Split along those same two verbs to stay under the line budget: this file
//! is discovery only, turning `slot(...)` calls into [`SlotSpec`]s. Coercion,
//! rendering, and the source rewrite itself — [`fill`], [`FillError`],
//! [`RawValue`] — live in `template_fill.rs` and are re-exported here so the
//! public API is unchanged.

use std::ops::Range;

use typst_edit::{Call, find_calls};

/// Coercion, rendering, and the source rewrite; split out to keep this file
/// under its line budget. See [`fill`].
#[path = "template_fill.rs"]
mod fill;
pub use fill::{FillError, RawValue, fill};

/// What kind of value a slot holds; drives how a filled value renders to Typst.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotKind {
    /// A quoted string, e.g. `"hello"`.
    String,
    /// An integer, e.g. `8`.
    Int,
    /// A float, e.g. `0.7`.
    Float,
    /// A real-scale number, e.g. `2.5`. Renders identically to [`Self::Float`];
    /// kept as its own variant so a numeric field's `ScaleKind` (spelled
    /// `int`/`real` in config) round-trips faithfully through a synthetic
    /// slot's kind, rather than reporting the generic `float` spelling.
    Real,
    /// A boolean, `true` or `false`.
    Bool,
    /// A `YYYY-MM-DD` date, rendered as `on(Y, M, D)`.
    Date,
    /// An array of strings, e.g. `("a", "b",)`.
    List,
    /// Raw markup spliced into the body; a markup `#slot(…)` drops its `#`.
    /// Escaped on the way in by the CLI (`add.rs::markup_safe`, via
    /// [`crate::status_value::escape_markup`]) since a filled value may be
    /// caller-supplied and untrusted. Compare [`Self::Typst`].
    Text,
    /// Raw Typst markup spliced into the body, same as [`Self::Text`] but
    /// never escaped. The trust decision belongs to the template author, not
    /// the caller: a template is checked-in repo content, a `--set`/`-f`/
    /// environment value is not, so `kind: "typst"` is the template's own
    /// assertion that this slot's filler is safe to splice verbatim. A filled
    /// value that fails to parse as Typst markup is rejected at [`fill`]
    /// time, naming the slot — never written to disk.
    Typst,
    /// A string constrained to the slot's `choices`.
    Choice,
}

impl SlotKind {
    /// The config/DSL spelling of this kind.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Int => "int",
            Self::Float => "float",
            Self::Real => "real",
            Self::Bool => "bool",
            Self::Date => "date",
            Self::List => "list",
            Self::Text => "text",
            Self::Typst => "typst",
            Self::Choice => "choice",
        }
    }

    fn parse(text: &str) -> Option<Self> {
        let kind = match text {
            "string" => Self::String,
            "int" => Self::Int,
            "float" => Self::Float,
            "real" => Self::Real,
            "bool" => Self::Bool,
            "date" => Self::Date,
            "list" => Self::List,
            "text" => Self::Text,
            "typst" => Self::Typst,
            "choice" => Self::Choice,
            _ => return None,
        };
        Some(kind)
    }

    /// Whether this kind splices its value straight into body markup —
    /// dropping a `#slot(...)` call's leading `#` and unquoting a
    /// string-literal default, rather than rendering as a self-delimiting
    /// Typst literal (a quoted string, a number, a list). `text` and `typst`
    /// both hold body content; they differ only in whether the value is
    /// escaped first, a decision made upstream of this crate.
    fn splices_verbatim(self) -> bool {
        matches!(self, Self::Text | Self::Typst)
    }
}

/// One discovered `slot(...)` call: its declared schema plus the source span to
/// replace when filling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotSpec {
    /// Slot name (first positional argument); the fill key.
    pub name: String,
    /// Value kind.
    pub kind: SlotKind,
    /// Whether a value must be supplied when no default is present.
    pub required: bool,
    /// Optional human prompt for interactive fill / `describe`.
    pub prompt: Option<String>,
    /// Optional environment variable name the CLI reads at fill time. `None`
    /// keeps the implicit `MINDTAPE_ADD_<NAME>` lookup.
    pub env: Option<String>,
    /// Allowed values for a `choice` slot.
    pub choices: Option<Vec<String>>,
    /// Default expression's raw source (spliced verbatim when unfilled). `None`
    /// for `default: none` or an absent `default`.
    pub default: Option<String>,
    /// Unescaped content when `default` is a string literal — used so a `text`
    /// slot's default lands in the body as markup, not as a quoted string.
    pub default_string: Option<String>,
    /// Whether the call is markup (`#slot(…)`), so a `text` fill drops the `#`.
    pub markup: bool,
    /// Byte span of the whole `slot(...)` call.
    pub range: Range<usize>,
}

/// Discover every `slot(...)` call in `source`, in document order.
#[must_use]
pub fn discover_slots(source: &str) -> Vec<SlotSpec> {
    find_calls(source, "slot")
        .iter()
        .filter_map(|call| spec_from_call(source, call))
        .collect()
}

fn spec_from_call(source: &str, call: &Call) -> Option<SlotSpec> {
    let name = call.positional(0)?.value.clone();
    let kind = call
        .named("kind")
        .and_then(|arg| SlotKind::parse(&arg.value))
        .unwrap_or(SlotKind::String);
    let required = call.named("required").is_none_or(|arg| arg.value == "true");
    let prompt = call.named("prompt").map(|arg| arg.value.clone());
    let env = call.named("env").map(|arg| arg.value.clone());
    let choices = call
        .named("choices")
        .map(|arg| parse_string_array(&arg.value));
    let default_arg = call
        .named("default")
        .filter(|arg| arg_raw(source, arg) != "none");
    let default = default_arg.map(|arg| arg_raw(source, arg).to_owned());
    let default_string = default_arg
        .filter(|arg| arg_raw(source, arg).starts_with('"'))
        .map(|arg| arg.value.clone());
    Some(SlotSpec {
        name,
        kind,
        required,
        prompt,
        env,
        choices,
        default,
        default_string,
        markup: call.markup,
        range: call.range.clone(),
    })
}

fn arg_raw<'a>(source: &'a str, arg: &typst_edit::Arg) -> &'a str {
    source.get(arg.value_range.clone()).unwrap_or_default()
}

/// Pull the string literals out of an array source like `("a", "b")`. A
/// literals-only scan: collects each `"…"` segment, honouring `\"` escapes.
fn parse_string_array(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = source.chars();
    while let Some(ch) = chars.next() {
        if ch != '"' {
            continue;
        }
        let mut current = String::new();
        let mut escaped = false;
        for inner in chars.by_ref() {
            if escaped {
                current.push(inner);
                escaped = false;
            } else if inner == '\\' {
                escaped = true;
            } else if inner == '"' {
                break;
            } else {
                current.push(inner);
            }
        }
        out.push(current);
    }
    out
}

#[cfg(test)]
#[path = "template_tests.rs"]
mod tests;
