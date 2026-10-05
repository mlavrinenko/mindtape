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
pub use fill::{FillError, RawValue, fill, fill_examples};

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
    /// caller-supplied and untrusted. Compare [`Self::Raw`].
    Text,
    /// Raw source spliced in verbatim, same as [`Self::Text`] but never
    /// escaped. Named for what it does rather than for the language it holds:
    /// the value is pasted, not read, so nothing here builds a Typst value
    /// and nothing type-checks one.
    ///
    /// The trust decision belongs to the template author, not the caller: a
    /// template is checked-in repo content, a `--set`/`-f`/environment value
    /// is not, so `kind: "raw"` is the template's own assertion that this
    /// slot's filler is safe to splice.
    ///
    /// What IS checked is narrow, and the name is honest about it. A value
    /// that fails to parse as Typst markup is rejected at [`fill`] time,
    /// naming the slot; that catches a markup-position slot, which is what
    /// the kind is for. It cannot catch a slot spliced into an argument
    /// position, where the value is an expression and parens are not markup —
    /// see `template_markup.rs`.
    Raw,
    /// A string constrained to the slot's `choices`.
    Choice,
    /// A list of strings, each one of the slot's `choices`.
    MultiChoice,
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
            Self::Raw => "raw",
            Self::Choice => "choice",
            Self::MultiChoice => "multichoice",
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
            "raw" => Self::Raw,
            "choice" => Self::Choice,
            "multichoice" => Self::MultiChoice,
            _ => return None,
        };
        Some(kind)
    }

    /// Whether this kind splices its value straight into body markup —
    /// dropping a `#slot(...)` call's leading `#` and unquoting a
    /// string-literal default, rather than rendering as a self-delimiting
    /// Typst literal (a quoted string, a number, a list). `text` and `raw`
    /// both hold body content; they differ only in whether the value is
    /// escaped first, a decision made upstream of this crate.
    fn splices_verbatim(self) -> bool {
        matches!(self, Self::Text | Self::Raw)
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
    /// Unescaped content when `default` is a string literal, or a raw block's
    /// text dedented as Typst dedents it — used so a `text` or `raw` slot's
    /// default lands in the body as markup, not as a quoted string.
    pub default_string: Option<String>,
    /// The `example:` argument's raw source: a sample value for a caller to
    /// read, never filed and never evaluated. `None` for `example: none` or an
    /// absent `example`.
    pub example: Option<String>,
    /// [`Self::example`]'s content when it is a string literal or a raw block,
    /// read the way [`Self::default_string`] reads a default.
    pub example_string: Option<String>,
    /// Whether the call is markup (`#slot(…)`), so a `text` fill drops the `#`.
    pub markup: bool,
    /// The `kind:` spelling when it names no known kind, e.g. a typo or a
    /// renamed kind a template has not caught up with. `None` when the kind
    /// parsed, or when the call declared none at all.
    ///
    /// Discovery cannot fail — it returns a `Vec`, and a template with one bad
    /// slot still has to describe its good ones — so the bad spelling is
    /// carried here and rejected by [`fill`] instead. Defaulting it to
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

#[cfg(test)]
#[path = "template_fill_tests.rs"]
mod tests_fill;
