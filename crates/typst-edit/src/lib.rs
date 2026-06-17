//! In-place editing for Typst source.
//!
//! A consumer-blind companion to the Typst parse plane: locate byte spans in
//! source (function-call arguments, string literals, links) and apply a set of
//! edits as one atomic, non-overlapping splice. No eval, no `World`, no IO — it
//! works on text and the [`typst_syntax`] parse tree alone, so the owning
//! document is the caller's concern.
//!
//! Two halves:
//! - [`find_calls`] locates any function call with its argument value spans and
//!   trailing content block; [`find_link_targets`] is the thin link-only case.
//! - [`apply`] rewrites a source string from a validated [`Edit`] set.
//!
//! Because edits are validated as a whole before anything is written, a rejected
//! set leaves no partial result — callers compose multi-file rewrites that abort
//! cleanly when any one file is wrong-shaped.
//!
//! ```
//! use typst_edit::{Edit, apply, find_link_targets};
//! let src = "see #link(\"old.typ\")[here]";
//! let targets = find_link_targets(src);
//! let target = targets.first().expect("one link");
//! let edit = Edit::new(target.range.clone(), "\"new.typ\"");
//! let out = apply(src, vec![edit]).expect("apply");
//! assert_eq!(out, "see #link(\"new.typ\")[here]");
//! ```

use std::fmt;
use std::ops::Range;

use typst_syntax::{LinkedNode, Source, SyntaxKind, ast};

/// A single replacement: swap the bytes in `range` for `replacement`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Edit {
    /// Byte range in the source to replace.
    pub range: Range<usize>,
    /// Text spliced in place of `range`.
    pub replacement: String,
}

impl Edit {
    /// Build an edit replacing `range` with `replacement`.
    #[must_use]
    pub fn new(range: Range<usize>, replacement: impl Into<String>) -> Self {
        Self {
            range,
            replacement: replacement.into(),
        }
    }
}

/// Why an [`apply`] call was rejected before any byte was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    /// Two edits cover overlapping bytes.
    Overlap {
        /// The earlier range.
        prev: Range<usize>,
        /// The later range that overlaps it.
        next: Range<usize>,
    },
    /// An edit range runs past the end of the source.
    OutOfBounds {
        /// The offending range.
        range: Range<usize>,
        /// Source length in bytes.
        len: usize,
    },
    /// An edit boundary falls inside a multi-byte char.
    NotCharBoundary {
        /// The offending byte offset.
        offset: usize,
    },
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overlap { prev, next } => {
                write!(f, "overlapping edits: {prev:?} and {next:?}")
            }
            Self::OutOfBounds { range, len } => {
                write!(f, "edit {range:?} out of bounds (source len {len})")
            }
            Self::NotCharBoundary { offset } => {
                write!(f, "edit boundary {offset} is not a char boundary")
            }
        }
    }
}

impl std::error::Error for EditError {}

/// Apply `edits` to `source`, returning the rewritten string.
///
/// Edits are sorted by start offset, then validated as a set: each range must lie
/// within the source on char boundaries, and ranges must not overlap. Validation
/// runs to completion before any byte is written, so a rejected set leaves no
/// partial result — the atomicity callers rely on. Untouched bytes are copied
/// verbatim.
///
/// # Errors
/// Returns [`EditError`] when an edit is out of bounds, splits a char, or overlaps
/// another edit.
pub fn apply(source: &str, mut edits: Vec<Edit>) -> Result<String, EditError> {
    edits.sort_by(|a, b| a.range.start.cmp(&b.range.start));
    validate(source, &edits)?;
    Ok(splice(source, &edits))
}

fn validate(source: &str, edits: &[Edit]) -> Result<(), EditError> {
    let len = source.len();
    let mut prev: Option<&Range<usize>> = None;
    for edit in edits {
        let range = &edit.range;
        if range.start > range.end || range.end > len {
            return Err(EditError::OutOfBounds {
                range: range.clone(),
                len,
            });
        }
        if !source.is_char_boundary(range.start) {
            return Err(EditError::NotCharBoundary {
                offset: range.start,
            });
        }
        if !source.is_char_boundary(range.end) {
            return Err(EditError::NotCharBoundary { offset: range.end });
        }
        if let Some(previous) = prev
            && range.start < previous.end
        {
            return Err(EditError::Overlap {
                prev: previous.clone(),
                next: range.clone(),
            });
        }
        prev = Some(range);
    }
    Ok(())
}

fn splice(source: &str, edits: &[Edit]) -> String {
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0;
    for edit in edits {
        if let Some(gap) = source.get(cursor..edit.range.start) {
            out.push_str(gap);
        }
        out.push_str(&edit.replacement);
        cursor = edit.range.end;
    }
    if let Some(tail) = source.get(cursor..) {
        out.push_str(tail);
    }
    out
}

/// A located function call, e.g. `link("…")[Label]`, `git-ref("p", "c")`, or
/// `scenario("name", stage: "wip")`.
///
/// Carries everything a rewriter needs without re-parsing: each argument's value
/// and byte range, and the trailing `[…]` content block. One [`find_calls`] pass
/// subsumes the textual span-scanning that consumers would otherwise hand-roll.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Call {
    /// Callee identifier (`link`, `git-ref`, `scenario`, …).
    pub name: String,
    /// Whole call span, including any trailing `[…]` content block.
    pub range: Range<usize>,
    /// Parenthesised arguments in source order; the content block is not here.
    pub args: Vec<Arg>,
    /// Trailing `[…]` content block, when present.
    pub body: Option<Body>,
    /// True when this is a markup `#name(...)` call (hash-prefixed); false for a
    /// call in code/expression context (a metadata-slot `link("…")`, an array
    /// element, a show-rule argument). Lets a consumer keep prose-only semantics
    /// — `#link` in body text vs. `link()` inside the task DSL — without a
    /// textual hash scan.
    pub markup: bool,
}

/// One argument of a [`Call`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Arg {
    /// `Some(key)` for a named argument `key: value`; `None` when positional.
    pub name: Option<String>,
    /// Unescaped text for a string-literal value; otherwise the value's source.
    pub value: String,
    /// Byte range of the value (quotes included for a string literal), ready for
    /// [`Edit::new`].
    pub value_range: Range<usize>,
}

/// The trailing `[…]` content block of a [`Call`] — typically a link label.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Body {
    /// Byte range of the whole block, brackets included.
    pub range: Range<usize>,
    /// Source between the brackets, verbatim.
    pub text: String,
}

impl Call {
    /// The `i`-th positional argument (named arguments skipped).
    #[must_use]
    pub fn positional(&self, i: usize) -> Option<&Arg> {
        self.args.iter().filter(|arg| arg.name.is_none()).nth(i)
    }

    /// The named argument `key`, if present.
    #[must_use]
    pub fn named(&self, key: &str) -> Option<&Arg> {
        self.args
            .iter()
            .find(|arg| arg.name.as_deref() == Some(key))
    }

    /// The call span without any trailing content block — the `name(args)` head.
    /// Replace this to swap the call while leaving its `[Label]` untouched.
    #[must_use]
    pub fn head_range(&self) -> Range<usize> {
        match &self.body {
            Some(body) => self.range.start..body.range.start,
            None => self.range.clone(),
        }
    }
}

/// Locate every call to the function named `name`, in document order.
///
/// Parses with [`typst_syntax`], so calls inside raw blocks or escaped
/// (`\#name`) are not function calls and never match — no textual masking needed.
#[must_use]
pub fn find_calls(source: &str, name: &str) -> Vec<Call> {
    let parsed = Source::detached(source);
    let root = LinkedNode::new(parsed.root());
    let mut out = Vec::new();
    collect_calls(source, &root, name, &mut out);
    out
}

fn collect_calls(src: &str, node: &LinkedNode, name: &str, out: &mut Vec<Call>) {
    if node.kind() == SyntaxKind::FuncCall
        && let Some(call) = call_at(src, node, name)
    {
        out.push(call);
    }
    for child in node.children() {
        collect_calls(src, &child, name, out);
    }
}

fn call_at(src: &str, call: &LinkedNode, name: &str) -> Option<Call> {
    let typed = call.get().cast::<ast::FuncCall>()?;
    let ast::Expr::Ident(callee) = typed.callee() else {
        return None;
    };
    if callee.as_str() != name {
        return None;
    }
    let args_node = call.children().find(|c| c.kind() == SyntaxKind::Args)?;
    let (args, body) = collect_args(src, &args_node);
    let markup = call
        .prev_sibling()
        .is_some_and(|sib| sib.kind() == SyntaxKind::Hash);
    Some(Call {
        name: name.to_owned(),
        range: call.range(),
        args,
        body,
        markup,
    })
}

fn collect_args(src: &str, args_node: &LinkedNode) -> (Vec<Arg>, Option<Body>) {
    let mut args = Vec::new();
    let mut body = None;
    for child in args_node.children() {
        match child.kind() {
            SyntaxKind::Named => {
                if let Some(arg) = named_arg(src, &child) {
                    args.push(arg);
                }
            }
            SyntaxKind::ContentBlock => body = Some(content_body(src, &child)),
            SyntaxKind::LeftParen | SyntaxKind::RightParen | SyntaxKind::Comma => {}
            kind if is_trivia(kind) => {}
            _ => {
                let (value, value_range) = value_of(src, &child);
                args.push(Arg {
                    name: None,
                    value,
                    value_range,
                });
            }
        }
    }
    (args, body)
}

fn named_arg(src: &str, node: &LinkedNode) -> Option<Arg> {
    // Children are `key : value` (with possible trivia). Split on the colon so a
    // bare-identifier value (`stage: wip`) is not mistaken for the key.
    let children: Vec<LinkedNode> = node.children().collect();
    let colon = children
        .iter()
        .position(|c| c.kind() == SyntaxKind::Colon)?;
    let key = children
        .get(..colon)?
        .iter()
        .find(|c| c.kind() == SyntaxKind::Ident)?
        .get()
        .text()
        .to_string();
    let value = children
        .get(colon + 1..)?
        .iter()
        .find(|c| !is_trivia(c.kind()))?;
    let (text, value_range) = value_of(src, value);
    Some(Arg {
        name: Some(key),
        value: text,
        value_range,
    })
}

fn value_of(src: &str, node: &LinkedNode) -> (String, Range<usize>) {
    let range = node.range();
    if node.kind() == SyntaxKind::Str
        && let Some(literal) = node.get().cast::<ast::Str>()
    {
        return (literal.get().as_str().to_owned(), range);
    }
    let text = src.get(range.clone()).unwrap_or_default().to_owned();
    (text, range)
}

fn content_body(src: &str, node: &LinkedNode) -> Body {
    let range = node.range();
    let inner = range.start.saturating_add(1)..range.end.saturating_sub(1);
    let text = src.get(inner).unwrap_or_default().to_owned();
    Body { range, text }
}

fn is_trivia(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Space | SyntaxKind::LineComment | SyntaxKind::BlockComment
    )
}

/// 1-based line number of byte offset `at` in `source`.
#[must_use]
pub fn line_of(source: &str, at: usize) -> usize {
    let head = source.get(..at).unwrap_or(source);
    1 + head.bytes().filter(|&b| b == b'\n').count()
}

/// A located `link("…")` target: its URL value and the byte range of the quoted
/// string literal (quotes included), ready to feed to [`Edit::new`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LinkTarget {
    /// The link target string, unquoted and unescaped.
    pub url: String,
    /// Byte range of the `"…"` literal in the source, quotes included.
    pub range: Range<usize>,
}

/// Locate every `link("…")` call in `source`, in document order.
///
/// A thin convenience over [`find_calls`] for the common case: keeps only links
/// whose target is a string literal, projecting each to its URL and value range.
#[must_use]
pub fn find_link_targets(source: &str) -> Vec<LinkTarget> {
    find_calls(source, "link")
        .into_iter()
        .filter_map(|call| {
            let arg = call
                .args
                .iter()
                .find(|arg| source.as_bytes().get(arg.value_range.start) == Some(&b'"'))?;
            Some(LinkTarget {
                url: arg.value.clone(),
                range: arg.value_range.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
