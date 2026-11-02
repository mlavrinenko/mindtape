//! Folding the Typst warnings a project load reports.
//!
//! Every command folds a warning with one message into a single line: its
//! count, the files it sits in, and each distinct hint once. `-v` keeps every
//! site.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use mindtape::LoadWarning;

use crate::diag::{Diag, Kind};

/// The line for a Typst warning on a file that loaded.
pub(crate) fn warning_diag(warning: &LoadWarning) -> Diag {
    at_place(
        Diag::verbatim(Kind::Warning, &warning.message),
        &warning.file,
        warning.line,
        warning.column,
    )
    .typst_hints(warning.hints.iter().cloned())
}

/// `diag` placed at `file:line:col`, or at the bare file when no line is known.
pub(crate) fn at_place(
    diag: Diag,
    file: &Path,
    line: Option<usize>,
    column: Option<usize>,
) -> Diag {
    match line {
        Some(line) => diag.at(file, line, column),
        None => diag.at_file(file),
    }
}

/// Fold warnings with the same Typst message into one line naming the count
/// and the files, each distinct hint once; `verbose` keeps every site.
pub(crate) fn fold_warnings(warnings: &[LoadWarning], verbose: bool) -> Vec<Diag> {
    if verbose {
        return warnings.iter().map(warning_diag).collect();
    }
    let mut order: Vec<&str> = Vec::new();
    let mut groups: BTreeMap<&str, Vec<&LoadWarning>> = BTreeMap::new();
    for warning in warnings {
        if let Some(group) = groups.get_mut(warning.message.as_str()) {
            group.push(warning);
        } else {
            order.push(&warning.message);
            groups.insert(&warning.message, vec![warning]);
        }
    }
    order
        .into_iter()
        .filter_map(|message| {
            let group = groups.get(message)?;
            Some(match group.as_slice() {
                [only] => warning_diag(only),
                _ => folded_warning(message, group),
            })
        })
        .collect()
}

/// One line for a message `warnings` repeat: its count, the files it sits in,
/// and every distinct hint Typst gave it.
fn folded_warning(message: &str, warnings: &[&LoadWarning]) -> Diag {
    let mut files = Vec::new();
    let mut seen = BTreeSet::new();
    for warning in warnings {
        let file = mindtape::display_path(&warning.file);
        if seen.insert(file.clone()) {
            files.push(file);
        }
    }
    let place = match files.as_slice() {
        [only] => only.clone(),
        _ => format!("{} files: {}", files.len(), files.join(", ")),
    };
    let mut hints = Vec::new();
    let mut seen_hints = BTreeSet::new();
    for warning in warnings {
        for hint in &warning.hints {
            if seen_hints.insert(hint.clone()) {
                hints.push(hint.clone());
            }
        }
    }
    Diag::verbatim(
        Kind::Warning,
        format!("{message}: {} times in {place}", warnings.len()),
    )
    .typst_hints(hints)
}

#[cfg(test)]
#[path = "warnings_tests.rs"]
mod tests;
