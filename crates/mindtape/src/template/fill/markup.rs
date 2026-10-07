//! Fill-time validation for `raw`-kind slots.
//!
//! `kind: "raw"` is the template author's own assertion that a slot's filler
//! is trusted content, so [`super::fill`] splices it verbatim — no escaping
//! (see [`super::SlotKind::Raw`]). Trusted does not mean well-formed, though:
//! a `--set`/`-f` value can still be broken markup, so this checks it parses
//! before it is ever spliced in, and names the slot in the error rather than
//! letting a bad value surface later as a bare compiler diagnostic about a
//! file the caller never hand-wrote.
//!
//! This check reads the value as top-level MARKUP, which is the plane a
//! `#slot(…)` in the body actually lands on. A `raw` slot spliced into an
//! ARGUMENT position is a different plane — there the value is an expression,
//! and markup treats parens and `*` as ordinary text, so `(files: 2` and
//! `*emphasis*` both parse clean here and reach the file. `mt add` then fails
//! loading what it just wrote. That is a known gap, not an oversight; the
//! kind is named `raw` because the splice, not the check, is the contract.
//!
//! `typst_syntax` is already a direct dependency of this crate (see
//! `framework/validate.rs`), pinned to the same 0.15 line
//! `typst-harvest`/`typstyle-core` resolve, so this adds no second `typst`
//! crate. Split into its own file to keep `template/mod.rs` under its line budget.

use super::{FillError, RawValue, SlotSpec, coerce_str};

/// Splice a `raw` slot's value verbatim, but only once it parses as Typst
/// markup. Catches syntax errors only (unclosed brackets, stray delimiters);
/// a semantic error (an undefined reference) still surfaces from the later
/// whole-document validation `mt add` already runs.
pub(super) fn render_raw(spec: &SlotSpec, value: &RawValue) -> Result<String, FillError> {
    let text = coerce_str(spec, value)?;
    if let Some(message) = first_markup_error(&text) {
        return Err(FillError::BadMarkup {
            slot: spec.name.clone(),
            message,
        });
    }
    Ok(text)
}

/// The first syntax error Typst's own parser reports for `text` read as
/// top-level markup, or `None` when it parses clean.
fn first_markup_error(text: &str) -> Option<String> {
    let node = typst_syntax::parse(text);
    let (errors, _warnings) = node.errors_and_warnings();
    errors.into_iter().next().map(|err| err.message.to_string())
}
