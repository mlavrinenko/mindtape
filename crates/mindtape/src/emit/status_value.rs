//! Render a status field value as Typst source.
//!
//! A status set from the CLI (`add`'s initial stamp, `flip`'s transition) is
//! written into `task.with(…)` as a call to a prelude constructor. A name the
//! prelude ships a shortcut for renders as the ergonomic `name(y, m, d)`; any
//! other name falls back to the generic `status("name", y, m, d)`. Centralising
//! the rule here keeps `add` and `flip` from each carrying a copy (the jscpd
//! copy-paste gate) and means the shortcut set lives in one place.
//!
//! [`escape_markup`] is the project's single Typst-markup escaper. It started
//! here for status notes, but every writer that splices caller text into a
//! content block faces the same hazard (`mt add`'s `text` slots), so it is
//! public and shared rather than copied — a second escaper would be a second
//! place for the hazard set to drift.

/// Status names the prelude (`lib/prelude.typ`) ships as shortcut constructors.
/// A legend value in this set renders as the ergonomic `name(y, m, d)`; any other
/// name (a project's custom status) falls back to `status("name", y, m, d)`. Keep
/// in sync with the `_mk(…)` bindings in the prelude; `shortcuts_match_prelude`
/// guards the drift.
pub const STATUS_SHORTCUTS: &[&str] = &[
    "draft",
    "proposed",
    "accepted",
    "todo",
    "wip",
    "doing",
    "blocked",
    "in-review",
    "done",
    "cancelled",
    "failed",
];

/// Render `name` dated `ymd` as a Typst status call, with an optional trailing
/// note. A shortcut name uses `name(y, m, d)` and carries the note as a trailing
/// content block (`done(y, m, d)[note]`); a non-shortcut name uses the generic
/// `status("name", y, m, d)` and carries the note as its `note:` named argument.
/// A blank note is dropped.
#[must_use]
pub fn status_value(name: &str, ymd: (i32, u8, u8), note: Option<&str>) -> String {
    let (year, month, day) = ymd;
    let note = note.map(str::trim).filter(|text| !text.is_empty());
    if STATUS_SHORTCUTS.contains(&name) {
        let mut out = format!("{name}({year}, {month}, {day})");
        if let Some(note) = note {
            out.push_str(&format!("[{}]", escape_markup(note)));
        }
        out
    } else if let Some(note) = note {
        format!(
            "status(\"{name}\", {year}, {month}, {day}, note: [{}])",
            escape_markup(note)
        )
    } else {
        format!("status(\"{name}\", {year}, {month}, {day})")
    }
}

/// Escape the Typst markup characters that would break a `[…]` content block so
/// `text` reads back literally. Backslash first, then the structural/markup
/// leaders: the delimiters that open a block (`[`, `]`), the code/math/emphasis
/// leaders (`#`, `$`, `*`, `_`, `` ` ``), and the two sigils that swallow the
/// word after them (`@` reference, `<` label). A closing `>` needs no escape —
/// it is only special after a `<` that is now escaped — but it survives the
/// round trip either way.
///
/// Every writer that splices caller text into markup must route through this:
/// `status`/`flip` notes and `add`'s `text` slots today. Unescaped, ordinary
/// prose either fails to load (`create-*star*` opens an unclosed strong) or,
/// worse, loads with the markup silently applied — `<a>` and `@at` evaluate to
/// a label and a reference and vanish from the value read back.
///
/// `add`'s `raw`-kind slots ([`crate::template::SlotKind::Raw`]) are the
/// one deliberate exception, not an oversight: escaping is a default for
/// values of unknown trust, and a `raw` slot is the *template*, not this
/// function, asserting its filler is trusted markup — see
/// `cmd/add/mod.rs::markup_safe`.
#[must_use]
pub fn escape_markup(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if matches!(
            ch,
            '\\' | '[' | ']' | '#' | '$' | '*' | '_' | '`' | '@' | '<'
        ) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
#[path = "status_value_tests.rs"]
mod tests;
