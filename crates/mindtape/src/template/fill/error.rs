//! [`FillError`]: why [`super::fill`] refused a value. Split out of
//! `fill/mod.rs` to keep that file under its line budget, the same seam
//! `markup.rs` took.

use std::fmt;

use super::SlotKind;

/// Why [`super::fill`] rejected a value.
#[derive(Debug, Clone, PartialEq)]
pub enum FillError {
    /// Required slots with no value and no default, by name.
    Missing(Vec<String>),
    /// A value does not coerce to the slot's kind.
    Type {
        /// Slot name.
        slot: String,
        /// Expected kind.
        want: SlotKind,
        /// The supplied value's shape.
        got: &'static str,
    },
    /// A `choice` value outside the declared `choices`.
    BadChoice {
        /// Slot name.
        slot: String,
        /// The rejected value.
        value: String,
        /// The allowed values.
        choices: Vec<String>,
    },
    /// A `date` value that is not `YYYY-MM-DD`.
    BadDate {
        /// Slot name.
        slot: String,
        /// The rejected value.
        value: String,
    },
    /// A slot declared a `kind:` that the code doesn't recognize.
    UnknownKind {
        /// Slot name.
        slot: String,
        /// The unrecognised spelling.
        kind: String,
    },
    /// A `raw` slot's value failed to parse as Typst markup.
    BadMarkup {
        /// Slot name.
        slot: String,
        /// The message the parser gives for the first error.
        message: String,
    },
    /// The rewrite changeset was invalid (overlap / bounds).
    Edit(String),
}

impl fmt::Display for FillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(names) => write!(
                f,
                "missing required {}: {}",
                crate::plural(names.len(), "slot"),
                names.join(", ")
            ),
            Self::Type {
                slot,
                want: SlotKind::List | SlotKind::MultiChoice,
                got,
            } => write!(
                f,
                "slot {slot:?} expects a list; use --set {slot}+=value (repeatable) or \
                 -f/--values with a JSON/YAML array, got {got}"
            ),
            Self::Type {
                slot,
                want: SlotKind::Links,
                got,
            } => write!(
                f,
                "slot {slot:?} expects links; use -f/--values with a list of \
                 kind, task and note mappings, got {got}"
            ),
            Self::Type { slot, want, got } => {
                write!(f, "slot {slot:?} expects {}, got {got}", want.as_str())
            }
            Self::BadChoice {
                slot,
                value,
                choices,
            } => write!(
                f,
                "slot {slot:?} value {value:?} is not one of: {}",
                choices.join(", ")
            ),
            Self::BadDate { slot, value } => {
                write!(f, "slot {slot:?} value {value:?} is not a YYYY-MM-DD date")
            }
            Self::UnknownKind { slot, kind } => write!(
                f,
                "slot {slot:?} declares unknown kind {kind:?}; expected one of: {}",
                SlotKind::spellings()
            ),
            Self::BadMarkup { slot, message } => {
                write!(f, "slot {slot:?} is not valid Typst markup: {message}")
            }
            Self::Edit(message) => write!(f, "rewrite failed: {message}"),
        }
    }
}

impl std::error::Error for FillError {}
