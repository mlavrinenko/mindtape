//! A [`Violation`] as one line: `severity: location: field: message`.
//!
//! `mt check`, a refused write and a flip all print a breach through here, so a
//! label always matches whether the breach fails the command. The words come
//! from the breach's [`Diag`].

use mindtape::Violation;

use crate::diag::Diag;

/// A breach as `severity: location: field: message`.
pub(crate) fn line(violation: &Violation) -> String {
    crate::diag::line(&Diag::from(violation))
}

/// A breach as `location: field: message`, for a message that carries its
/// severity elsewhere (a joined problem list, a summary line).
pub(crate) fn at(violation: &Violation) -> String {
    Diag::from(violation).to_string()
}
