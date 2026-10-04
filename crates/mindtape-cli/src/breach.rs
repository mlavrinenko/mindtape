//! The one renderer for a [`Violation`]: `severity: location: field: message`.
//!
//! `mt check`, a refused write and a flip all print a breach through here, so a
//! label always matches whether the breach fails the command.

use mindtape::Violation;

/// A breach as `severity: location: field: message`.
pub(crate) fn line(violation: &Violation) -> String {
    format!("{}: {}", violation.severity.label(), at(violation))
}

/// A breach as `location: field: message`, for a message that carries its
/// severity elsewhere (a joined problem list, a summary line).
pub(crate) fn at(violation: &Violation) -> String {
    format!(
        "{}: {}: {}",
        violation.location, violation.field, violation.message
    )
}
