//! Slot-template engine: discover `slot(...)` calls and fill them by source
//! rewrite.
//!
//! A template is an ordinary task artifact whose authored values are `slot(...)`
//! calls (see the prelude helper). This module finds each call on the parse plane
//! (`typst_edit::find_calls`, with no eval and no `World`) and reads its literal
//! arguments into a [`SlotSpec`]. [`fill()`] then rewrites the source: it
//! replaces each slot with a rendered literal. The filed task holds plain values
//! and never calls `slot`.
//!
//! Slot arguments must be literals. This engine never evaluates the template.
//!
//! The split follows those same two verbs to stay under the line budget. This
//! file holds discovery only, turning `slot(...)` calls into [`SlotSpec`]s.
//! Coercion, rendering, and the source rewrite ([`fill()`], [`FillError`],
//! [`RawValue`]) live in `template/fill/mod.rs`, and this file re-exports them
//! to keep the public API the same.
//!
//! # Files
//!
//! - `add_slots.rs`: synthetic `slot(...)` calls for config-required core fields.
//! - `check.rs`: what `mt check` makes of a template.
//! - `fill/`: slot fill, coercing caller-supplied values against the discovered slots.

pub mod add_slots;
pub mod check;

use std::ops::Range;

use typst_edit::{Call, find_calls};

/// Coercion, rendering, and the source rewrite, split out to keep this file
/// under its line budget. See [`fill()`].
mod fill;
pub use fill::{FillError, LinkValue, RawValue, fill, fill_defaults, fill_examples, fill_partial};

/// The kind of value a slot holds. It decides how a filled value renders to
/// Typst.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotKind {
    /// A quoted string, such as `"hello"`.
    String,
    /// An integer, such as `8`.
    Int,
    /// A float, such as `0.7`.
    Float,
    /// A real-scale number, such as `2.5`. Renders identically to
    /// [`Self::Float`]. It stays a separate variant so a numeric field's
    /// `ScaleKind` (spelled `int`/`real` in config) round-trips faithfully
    /// through a synthetic slot's kind, rather than reporting the generic
    /// `float` spelling.
    Real,
    /// A boolean, `true` or `false`.
    Bool,
    /// A `YYYY-MM-DD` date, rendered as `on(Y, M, D)`.
    Date,
    /// An array of strings, such as `("a", "b",)`.
    List,
    /// Raw markup spliced into the body. A markup `#slot(…)` drops its `#`.
    /// Escaped on the way in by the CLI (`cmd/add/mod.rs::markup_safe`, via
    /// [`crate::emit::status_value::escape_markup`]) since a filled value may be
    /// caller-supplied and untrusted. Compare [`Self::Raw`].
    Text,
    /// Raw source spliced in verbatim, same as [`Self::Text`] but never
    /// escaped. Named for what it does rather than for the language it holds:
    /// the engine pastes the value rather than reading it. This module only
    /// splices text: it never builds a Typst value and never type-checks one.
    ///
    /// The template author owns the trust decision, not the caller. A template
    /// is repo content under version control, while a `--set`/`-f`/environment
    /// value is not. So `kind: "raw"` is the template's own assertion that this
    /// slot's filler is safe to splice.
    ///
    /// The check stays narrow, and the name says so. [`fill()`] rejects a value
    /// that fails to parse as Typst markup, naming the slot. That catches a
    /// markup-position slot, which is what the kind is for. It cannot catch a
    /// slot spliced into an argument position, where the value is an expression
    /// and parens are not markup. See `template/fill/markup.rs`.
    Raw,
    /// A string constrained to the slot's `choices`.
    Choice,
    /// A list of strings, each one of the slot's `choices`.
    MultiChoice,
    /// A list of typed edges, each a `task`, `kind` and optional `note`,
    /// rendered as the dicts the `links:` helpers of the DSL build.
    Links,
}

impl SlotKind {
    /// Every kind once, in the order a refusal and the docs list them. `parse`
    /// reads the spelling of each from here, so this list is the full set of
    /// kinds.
    pub const ALL: [Self; 12] = [
        Self::String,
        Self::Int,
        Self::Float,
        Self::Real,
        Self::Bool,
        Self::Date,
        Self::List,
        Self::Text,
        Self::Raw,
        Self::Choice,
        Self::MultiChoice,
        Self::Links,
    ];

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
            Self::Raw => "raw",
            Self::Choice => "choice",
            Self::MultiChoice => "multichoice",
            Self::Links => "links",
        }
    }

    /// The spelling of each kind, comma-joined, as a refusal lists them.
    #[must_use]
    pub fn spellings() -> String {
        let names: Vec<_> = Self::ALL.iter().map(|kind| kind.as_str()).collect();
        names.join(", ")
    }

    fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == text)
    }

    /// Whether this kind splices its value straight into body markup. Such a
    /// kind drops a `#slot(...)` call's leading `#` and unquotes a
    /// string-literal default, rather than rendering a self-delimiting Typst
    /// literal (a quoted string, a number, a list). `text` and `raw` both hold
    /// body content, and they differ only in whether the CLI escapes the value
    /// first, a decision made upstream of this crate.
    fn splices_verbatim(self) -> bool {
        matches!(self, Self::Text | Self::Raw)
    }
}

/// One discovered `slot(...)` call: its declared schema plus the source span to
/// replace when filling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotSpec {
    /// Slot name (first positional argument), used as the fill key.
    pub name: String,
    /// Value kind.
    pub kind: SlotKind,
    /// Whether the caller must supply a value when no default is present.
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
    /// Unescaped content when `default` is a string literal, or a raw block's
    /// text dedented as Typst dedents it. It lets a `text` or `raw` slot's
    /// default land in the body as markup, not as a quoted string.
    pub default_string: Option<String>,
    /// The raw source of the `example:` argument, a sample value for a caller
    /// to read. It is never filed and never evaluated. `None` for
    /// `example: none` or an absent `example`.
    pub example: Option<String>,
    /// [`Self::example`]'s content when it is a string literal or a raw block,
    /// read the way [`Self::default_string`] reads a default.
    pub example_string: Option<String>,
    /// Whether the call is markup (`#slot(…)`), so a `text` fill drops the `#`.
    pub markup: bool,
    /// The `kind:` spelling when the template gives an unknown kind, such as a
    /// typo or a renamed kind that the template has not caught up with. `None`
    /// when the kind parsed, or when the call declared none.
    ///
    /// Discovery cannot fail: it returns a `Vec`, and a template with one bad
    /// slot still has to describe its good ones. So this field carries the bad
    /// spelling and [`fill()`] rejects it instead. Defaulting it to
    /// [`SlotKind::String`] silently, as this once did, turns a renamed kind
    /// into a quoted, escaped string with no diagnostic anywhere.
    pub unknown_kind: Option<String>,
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
    let declared = call.named("kind");
    let kind = declared
        .and_then(|arg| SlotKind::parse(&arg.value))
        .unwrap_or(SlotKind::String);
    let unknown_kind = declared
        .filter(|arg| SlotKind::parse(&arg.value).is_none())
        .map(|arg| arg.value.clone());
    let required = call.named("required").is_none_or(|arg| arg.value == "true");
    let prompt = call.named("prompt").map(|arg| arg.value.clone());
    let env = call.named("env").map(|arg| arg.value.clone());
    let choices = call
        .named("choices")
        .map(|arg| parse_string_array(&arg.value));
    let (default, default_string) = literal(source, call, "default");
    let (example, example_string) = literal(source, call, "example");
    Some(SlotSpec {
        name,
        kind,
        required,
        prompt,
        env,
        choices,
        default,
        default_string,
        example,
        example_string,
        markup: call.markup,
        unknown_kind,
        range: call.range.clone(),
    })
}

/// The named literal argument `key` of a slot call: its raw source, and its
/// content when it is a string literal or a raw block. `none` is no literal.
fn literal(source: &str, call: &Call, key: &str) -> (Option<String>, Option<String>) {
    let Some(arg) = call.named(key).filter(|arg| arg_raw(source, arg) != "none") else {
        return (None, None);
    };
    let raw = arg_raw(source, arg);
    let text = if raw.starts_with('"') {
        Some(arg.value.clone())
    } else {
        raw_block_text(raw)
    };
    (Some(raw.to_owned()), text)
}

/// A raw block's text, its lines dedented as Typst dedents them, whatever its
/// language tag. `None` when `source` is not a raw block.
fn raw_block_text(source: &str) -> Option<String> {
    if !source.starts_with('`') {
        return None;
    }
    let root = typst_syntax::parse_code(source);
    let raw = root
        .children()
        .find_map(typst_syntax::SyntaxNode::cast::<typst_syntax::ast::Raw>)?;
    let lines: Vec<&str> = raw.lines().map(|line| line.get().as_str()).collect();
    Some(lines.join("\n"))
}

fn arg_raw<'a>(source: &'a str, arg: &typst_edit::Arg) -> &'a str {
    source.get(arg.value_range.clone()).unwrap_or_default()
}

/// Pull the string literals out of an array source like `("a", "b")`. The scan
/// reads literals only, collecting each `"…"` segment and honouring `\"`
/// escapes. Public because a `+=` on a list slot appends to the slot's
/// `default`, which discovery leaves as raw source.
#[must_use]
pub fn parse_string_array(source: &str) -> Vec<String> {
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
mod tests;

#[cfg(test)]
mod tests_fill;

#[cfg(test)]
mod tests_fill_errors;
