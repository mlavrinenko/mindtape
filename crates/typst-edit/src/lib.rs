//! In-place editing for Typst source.
//!
//! A consumer-blind companion to the Typst parse plane: locate byte spans in
//! source (function-call arguments, string literals, links) and apply a set of
//! edits as one atomic, non-overlapping splice. No eval, no `World`, no IO — it
//! works on text and the [`typst_syntax`] parse tree alone, so the owning
//! document is the caller's concern.
//!
//! Two halves:
//! - [`find_link_targets`] locates spans over the parse tree.
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
/// Parses with [`typst_syntax`], so links inside raw blocks or escaped (`\#link`)
/// are not function calls and never match — no textual masking needed.
#[must_use]
pub fn find_link_targets(source: &str) -> Vec<LinkTarget> {
    let parsed = Source::detached(source);
    let root = LinkedNode::new(parsed.root());
    let mut out = Vec::new();
    collect_links(&root, &mut out);
    out
}

fn collect_links(node: &LinkedNode, out: &mut Vec<LinkTarget>) {
    if node.kind() == SyntaxKind::FuncCall
        && let Some(target) = link_target(node)
    {
        out.push(target);
    }
    for child in node.children() {
        collect_links(&child, out);
    }
}

fn link_target(call: &LinkedNode) -> Option<LinkTarget> {
    let typed = call.get().cast::<ast::FuncCall>()?;
    let ast::Expr::Ident(callee) = typed.callee() else {
        return None;
    };
    if callee.as_str() != "link" {
        return None;
    }
    let args = call.children().find(|c| c.kind() == SyntaxKind::Args)?;
    let literal = args.children().find(|c| c.kind() == SyntaxKind::Str)?;
    let value = literal.get().cast::<ast::Str>()?;
    Some(LinkTarget {
        url: value.get().as_str().to_owned(),
        range: literal.range(),
    })
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
