//! Normalize task Typst source to typstyle's canonical form.
//!
//! `mt` writes task files by surgical means only: `typst-edit` byte splices,
//! parse-plane slot fills, and [`crate::emit_task`]. Each of them leaves
//! formatting alone. A fill or an injected argument can leave non-canonical
//! whitespace behind (observed: a body carrying hundreds of leading spaces from
//! the original `mt add` write). [`format_typst`] runs the result through
//! typstyle, and every written file lands in one canonical shape.
//!
//! typstyle is a whole-document pretty printer. It normalizes indentation and
//! structure but does not reflow prose paragraphs, and it is idempotent: it
//! returns an already-clean file unchanged. Formatting on a surgical edit only
//! ever cleans drift, and never churns a canonical file beyond the edited span.
//!
//! Formatting is cosmetic, so it is error-tolerant. Malformed source (which `mt`
//! never writes, but a hand-broken file might carry into an edit) passes through
//! verbatim rather than failing the command. Correctness lives in validation,
//! not here.

use typstyle_core::Typstyle;

/// Reformat `source` into typstyle's canonical Typst form. Idempotent. Returns
/// `source` unchanged when typstyle cannot parse it (formatting is cosmetic and
/// never a correctness gate).
#[must_use]
pub fn format_typst(source: &str) -> String {
    Typstyle::default()
        .format_text(source)
        .render()
        .unwrap_or_else(|_| source.to_owned())
}

#[cfg(test)]
#[path = "format_tests.rs"]
mod tests;
