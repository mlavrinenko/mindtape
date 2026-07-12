//! `links:` field emission: group edges by kind, then by note, into one
//! prelude helper call per (kind, note) group, joined with ` + ` — the
//! DSL-call-level mirror of `lib/prelude.typ`'s own `_links-by-kind`
//! grouping (which operates on the already-evaluated marker for the rendered
//! card). Edges of the same kind but different notes need separate calls,
//! since a helper's trailing content block applies its note to every target
//! in that one call — so a kind can legally appear more than once in the
//! joined expression.

use crate::link::{Link, is_builtin_kind};

use super::format::typ_str;

/// The whole `links: …` field, or `None` when `links` is empty.
#[must_use]
pub(super) fn emit_links(links: &[Link]) -> Option<String> {
    if links.is_empty() {
        return None;
    }
    let mut kinds: Vec<&str> = Vec::new();
    for link in links {
        if !kinds.contains(&link.kind.as_str()) {
            kinds.push(link.kind.as_str());
        }
    }
    let calls: Vec<String> = kinds
        .into_iter()
        .flat_map(|kind| kind_calls(kind, links))
        .collect();
    Some(format!("links: {}", calls.join(" + ")))
}

/// One kind's helper calls: grouped by note (first-seen order), so a kind
/// whose edges carry different notes emits one call per distinct note.
fn kind_calls(kind: &str, links: &[Link]) -> Vec<String> {
    let mut notes: Vec<Option<&str>> = Vec::new();
    for link in links.iter().filter(|link| link.kind == kind) {
        let note = link.note.as_deref();
        if !notes.contains(&note) {
            notes.push(note);
        }
    }
    notes
        .into_iter()
        .map(|note| {
            let targets: Vec<&str> = links
                .iter()
                .filter(|link| link.kind == kind && link.note.as_deref() == note)
                .map(|link| link.task.as_str())
                .collect();
            call(kind, &targets, note)
        })
        .collect()
}

/// One helper call: the built-in name directly (`depends-on(...)`), else the
/// generic `edge("kind", ...)` escape hatch; a trailing `[note]` content
/// block when `note` is set.
fn call(kind: &str, targets: &[&str], note: Option<&str>) -> String {
    let args = targets
        .iter()
        .map(|target| typ_str(target))
        .collect::<Vec<_>>()
        .join(", ");
    let mut out = if is_builtin_kind(kind) {
        format!("{kind}({args})")
    } else {
        format!("edge({}, {args})", typ_str(kind))
    };
    if let Some(note) = note {
        out.push('[');
        out.push_str(&escape_note(note));
        out.push(']');
    }
    out
}

/// Escape characters that would otherwise break out of a Typst content block
/// (`[...]`) or trigger markup, so a note round-trips to the exact authored
/// text through `typst_harvest`'s plain-text projection.
fn escape_note(text: &str) -> String {
    const BREAKING: &str = "\\[]#$*_`@<";
    text.chars()
        .fold(String::with_capacity(text.len()), |mut acc, ch| {
            if BREAKING.contains(ch) {
                acc.push('\\');
            }
            acc.push(ch);
            acc
        })
}
