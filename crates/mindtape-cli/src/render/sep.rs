//! Separator policy for machine-readable output.
//!
//! When piping to another tool or writing CSV/JSON, ANSI colour must not
//! appear and field separators must be stable. This module centralises
//! those decisions.

/// Separator characters used for machine-readable text output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Separators {
    /// Character between fields on one line (e.g. `\t` for TSV).
    pub field: char,
    /// Character between records (e.g. `\n`).
    pub record: char,
}

/// Standard tab-separated-value separators for piped/machine output.
pub const TSV: Separators = Separators {
    field: '\t',
    record: '\n',
};

/// Comma-separated-value separators (field sep only; records still end in `\n`).
pub const CSV: Separators = Separators {
    field: ',',
    record: '\n',
};

/// Returns `true` if output is going to a machine consumer (not a TTY) and
/// structured separators are therefore preferred over decorative ones.
#[must_use]
pub fn is_machine_output() -> bool {
    use std::io::IsTerminal;
    !std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::{CSV, Separators, TSV, is_machine_output};

    #[test]
    fn tsv_separator_values() {
        assert_eq!(TSV.field, '\t');
        assert_eq!(TSV.record, '\n');
    }

    #[test]
    fn csv_separator_values() {
        assert_eq!(CSV.field, ',');
        assert_eq!(CSV.record, '\n');
    }

    #[test]
    fn separators_equality() {
        assert_eq!(
            TSV,
            Separators {
                field: '\t',
                record: '\n'
            }
        );
    }

    #[test]
    fn is_machine_output_true_in_tests() {
        // Test processes run without a real TTY, so stdout is not a terminal.
        assert!(is_machine_output());
    }
}
