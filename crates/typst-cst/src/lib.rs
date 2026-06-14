//! Parse-plane structure extraction for Typst.
//!
//! The syntactic counterpart to `typst-harvest`: where the harvester evaluates a
//! file to collect computed `metadata()` markers, this parses the source with
//! [`typst_syntax::parse`] (no eval, no `World`, no IO) and extracts the heading
//! structure as written — depth, text, and source span — so tools can gate the
//! shape of a document cheaply.
//!
//! ```
//! use typst_cst::{parse_headings, tree};
//! let src = "= Top\n== Sub\n";
//! let flat = parse_headings(src);
//! assert_eq!(flat.len(), 2);
//! assert_eq!(flat[0].depth, 1);
//! assert_eq!(flat[1].text, "Sub");
//! let roots = tree(flat);
//! assert_eq!(roots[0].children.len(), 1);
//! ```

use std::iter::Peekable;
use std::ops::Range;

use typst_syntax::{LinkedNode, Source, SyntaxKind};

/// A section heading as written in the source.
///
/// Line and byte span are consumer-blind: this crate works on text alone, so the
/// owning file is the caller's concern.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Heading {
    /// Section depth — the number of `=` markers (level 1 is `=`).
    pub depth: usize,
    /// The heading body text, trimmed (markup verbatim, e.g. `Phase 1: Auth`).
    pub text: String,
    /// One-based line of the heading marker in the source.
    pub line: usize,
    /// Byte range of the whole heading within the source text.
    pub range: Range<usize>,
}

/// A heading and the headings nested beneath it (deeper depth).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct HeadingNode {
    /// The heading at this node.
    pub heading: Heading,
    /// Headings directly nested under it.
    pub children: Vec<HeadingNode>,
}

/// Extract every heading from Typst source, in document order.
#[must_use]
pub fn parse_headings(text: &str) -> Vec<Heading> {
    let source = Source::detached(text);
    let root = LinkedNode::new(source.root());
    let mut out = Vec::new();
    collect(&root, &source, &mut out);
    out
}

/// Build the nesting tree from a document-order heading list.
///
/// A heading nests under the nearest preceding heading of smaller depth; depth
/// jumps (e.g. `=` then `===`) are tolerated.
#[must_use]
pub fn tree(headings: Vec<Heading>) -> Vec<HeadingNode> {
    let mut it = headings.into_iter().peekable();
    build(&mut it, 0)
}

fn build(it: &mut Peekable<impl Iterator<Item = Heading>>, min_depth: usize) -> Vec<HeadingNode> {
    let mut out = Vec::new();
    while let Some(next) = it.peek() {
        let depth = next.depth;
        if depth < min_depth {
            break;
        }
        let Some(heading) = it.next() else { break };
        let children = build(it, depth + 1);
        out.push(HeadingNode { heading, children });
    }
    out
}

fn collect(node: &LinkedNode, source: &Source, out: &mut Vec<Heading>) {
    if node.kind() == SyntaxKind::Heading
        && let Some(heading) = extract(node, source)
    {
        out.push(heading);
    }
    for child in node.children() {
        collect(&child, source, out);
    }
}

fn extract(node: &LinkedNode, source: &Source) -> Option<Heading> {
    let marker = node
        .children()
        .find(|c| c.kind() == SyntaxKind::HeadingMarker)?;
    let marker_range = marker.range();
    let range = node.range();
    let depth = marker_range.end - marker_range.start;
    let text = source
        .text()
        .get(marker_range.end..range.end)
        .unwrap_or_default()
        .trim()
        .to_owned();
    let line = source
        .lines()
        .byte_to_line(range.start)
        .map_or(0, |l| l + 1);
    Some(Heading {
        depth,
        text,
        line,
        range,
    })
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
