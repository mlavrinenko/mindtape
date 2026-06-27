//! Render a status field value as Typst source.
//!
//! A status set from the CLI (`add`'s initial stamp, `flip`'s transition) is
//! written into `task.with(…)` as a call to a prelude constructor. A name the
//! prelude ships a shortcut for renders as the ergonomic `name(y, m, d)`; any
//! other name falls back to the generic `status("name", y, m, d)`. Centralising
//! the rule here keeps `add` and `flip` from each carrying a copy (the jscpd
//! copy-paste gate) and means the shortcut set lives in one place.

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
            out.push_str(&format!("[{}]", escape_content(note)));
        }
        out
    } else if let Some(note) = note {
        format!(
            "status(\"{name}\", {year}, {month}, {day}, note: [{}])",
            escape_content(note)
        )
    } else {
        format!("status(\"{name}\", {year}, {month}, {day})")
    }
}

/// Escape the Typst markup characters that would break a `[…]` content block so a
/// note reads literally. Backslash first, then the structural/markup leaders.
fn escape_content(text: &str) -> String {
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
