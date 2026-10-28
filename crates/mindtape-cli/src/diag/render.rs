//! The renderer: a [`Diag`] to its lines, and the one writer to stderr.

use std::io::{IsTerminal, Write};
use std::process::ExitCode;

use anstyle::{AnsiColor, Color, Style};

use super::{Diag, Kind, Note, Tally};
use crate::render::color::{self, Painter};

/// `value` in single quotes, the way a shell takes it back: an inner `'`
/// becomes `'\''`.
pub(crate) fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

/// `word` as a shell reads it back: bare when every character is safe there,
/// else [`quote`]d.
pub(crate) fn shell_word(word: &str) -> String {
    shell_word_on(word, cfg!(windows))
}

/// [`shell_word`], `windows` saying whether `\` is safe bare: cmd and
/// `PowerShell` read it literally, so a Windows path needs no quotes.
pub(crate) fn shell_word_on(word: &str, windows: bool) -> String {
    let safe =
        |c: char| c.is_ascii_alphanumeric() || "-_./:@%+=,".contains(c) || (windows && c == '\\');
    if !word.is_empty() && word.chars().all(safe) {
        word.to_owned()
    } else {
        quote(word)
    }
}

/// The message line without its prefix: the place, the rule, the text with
/// its value and context, then the reason, joined with `: `. A trailing period
/// on the reason is dropped.
pub(super) fn message(diag: &Diag) -> String {
    let shown = |path: &std::path::Path| mindtape::display_path(path);
    let mut parts: Vec<String> = diag.0.place.iter().map(|place| place.text(shown)).collect();
    parts.extend(diag.0.rule.clone());
    parts.push(text(diag));
    parts.retain(|part| !part.is_empty());
    parts.join(": ")
}

/// What the problem says, without its place or rule: the text with its value
/// and context, then the reason. A trailing period on the reason is dropped.
pub(super) fn text(diag: &Diag) -> String {
    let mut parts = vec![head(diag)];
    if let Some(reason) = &diag.0.reason {
        parts.push(reason.strip_suffix('.').unwrap_or(reason).to_owned());
    }
    parts.retain(|part| !part.is_empty());
    parts.join(": ")
}

/// The text with its tally, value and context.
fn head(diag: &Diag) -> String {
    let mut head = diag.0.what.to_owned();
    if let Some(tally) = diag.0.tally {
        head.push_str(&tally_text(tally));
    }
    if !diag.0.values.is_empty() {
        let quoted: Vec<String> = diag.0.values.iter().map(|value| quote(value)).collect();
        head.push_str(": ");
        head.push_str(&quoted.join(", "));
    }
    if let Some((label, value)) = &diag.0.context {
        head.push(' ');
        head.push_str(label);
        head.push(' ');
        head.push_str(&quote(value));
    }
    head
}

/// ` 1 of 3 tasks; 2 failed`, the failed count only when some landed.
fn tally_text(tally: Tally) -> String {
    let failed = tally.total.saturating_sub(tally.done);
    let mut text = format!(
        " {} of {}",
        tally.done,
        mindtape::count(tally.total, "task")
    );
    if tally.done > 0 && failed > 0 {
        text.push_str(&format!("; {failed} failed"));
    }
    text
}

/// The message line with its prefix and no colour, for a breach printed among
/// other output.
pub(crate) fn line(diag: &Diag) -> String {
    format!("{}: {}", diag.0.kind.label(), message(diag))
}

/// Every line of `diag`, each ending in a newline and starting with its
/// prefix. A message line past the first, from an unconverted multi-line
/// message, becomes a note.
pub(super) fn render(diag: &Diag, colored: bool) -> String {
    let painter = Painter::new(colored);
    let mut out = String::new();
    let message = message(diag);
    let mut lines = message.lines();
    push(
        &mut out,
        painter,
        diag.0.kind.label(),
        lines.next().unwrap_or(""),
    );
    let continued = lines
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(Note::legacy);
    for note in continued {
        push(&mut out, painter, "note", &note.to_string());
    }
    for hint in &diag.0.hints {
        push(&mut out, painter, "hint", &hint.to_string());
    }
    for note in &diag.0.notes {
        push(&mut out, painter, "note", &note.to_string());
    }
    out
}

fn push(out: &mut String, painter: Painter, label: &str, text: &str) {
    out.push_str(&painter.paint(&format!("{label}:"), style(label)));
    out.push(' ');
    out.push_str(text);
    out.push('\n');
}

fn style(label: &str) -> Style {
    let bold = Style::new().bold();
    let color = match label {
        "error" => AnsiColor::Red,
        "warning" => AnsiColor::Yellow,
        "hint" => AnsiColor::Cyan,
        _ => return bold,
    };
    bold.fg_color(Some(Color::Ansi(color)))
}

/// Write `diag` to stderr. A failed write is dropped: there's nowhere left to
/// report it.
pub(crate) fn emit(diag: &Diag) {
    let text = render(diag, color::is_stderr_color_active());
    let _ = std::io::stderr().lock().write_all(text.as_bytes());
}

/// Write one `note:` line to stderr, for context that stands alone because
/// nothing failed or the failure is reported elsewhere.
pub(crate) fn emit_note(note: &Note) {
    let painter = Painter::new(color::is_stderr_color_active());
    let mut out = String::new();
    push(&mut out, painter, "note", &note.to_string());
    let _ = std::io::stderr().lock().write_all(out.as_bytes());
}

/// Write `text` to stderr as it is: a report that is not a message, such as
/// the per-record lines of `mt import`. A failed write is dropped.
pub(crate) fn emit_text(text: &str) {
    let _ = std::io::stderr().lock().write_all(text.as_bytes());
}

/// Whether stderr is a terminal, for the colour decision. Asked here, so
/// stderr has one owner.
pub(crate) fn stderr_is_terminal() -> bool {
    std::io::stderr().is_terminal()
}

/// Write `diag` to stderr and return the exit code its kind decides.
pub(crate) fn report(diag: &Diag) -> ExitCode {
    emit(diag);
    exit_code(diag.0.kind)
}

fn exit_code(kind: Kind) -> ExitCode {
    ExitCode::from(kind.code())
}
