//! The one renderer for a [`Violation`]: `severity: file:line: field: message`.
//!
//! `mt check`, a refused write and a flip all print a breach through here, so a
//! label always matches whether the breach fails the command.

use mindtape::Violation;

/// A breach as `severity: file:line: field: message`.
pub(crate) fn line(violation: &Violation) -> String {
    format!("{}: {}", violation.severity.label(), at(violation))
}

/// A breach as `file:line: field: message`, for a message that carries its
/// severity elsewhere (a joined problem list, a summary line).
pub(crate) fn at(violation: &Violation) -> String {
    let loc = &violation.location;
    format!(
        "{}:{}: {}: {}",
        loc.file.display(),
        loc.line,
        violation.field,
        violation.message
    )
}
