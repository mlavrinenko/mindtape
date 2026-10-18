//! The per-file failures and Typst warnings a project load reports.

use std::path::{Path, PathBuf};

use typst_harvest::HarvestError;

use crate::model::task_spans::{CallRegion, TaskCallSpans};

/// A per-file failure encountered while loading the project.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoadError {
    /// File that failed, relative to the project root when possible.
    pub file: PathBuf,
    /// One-based line of `file` an eval error points at: the first point of a
    /// diagnostic inside the file, its trace walked when Typst raised it in
    /// the prelude, else the file's `task.with(…)` call. Never a prelude line.
    /// `None` when the failure is not an eval error or has no place at all.
    pub line: Option<usize>,
    /// One-based column on `line`. `None` when `line` is, or when the line is
    /// the fallback `task.with(…)` call, which has no point of its own.
    pub column: Option<usize>,
    /// Human-readable cause: Typst's own message for an eval error.
    pub message: String,
    /// Typst's hints on the diagnostic that gave the place, in its order and
    /// wording. Empty when the failure is not an eval error.
    pub hints: Vec<String>,
    /// Where `line` and `column` lie against the file's `task.with(…)` call:
    /// in one argument's value, in the call, or outside it, which is also the
    /// region of a failure with no place. Boxed, so a `Result` that carries a
    /// `LoadError` stays under clippy's `result_large_err` size.
    pub region: Box<CallRegion>,
}

/// A Typst warning raised while evaluating a task file, whether or not the
/// file went on to load.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LoadWarning {
    /// File the warning was raised for, relative to the project root.
    pub file: PathBuf,
    /// One-based line of `file` the warning points at, when it points into it.
    pub line: Option<usize>,
    /// One-based column on `line`.
    pub column: Option<usize>,
    /// Typst's own message.
    pub message: String,
    /// Typst's hints, in its order and wording.
    pub hints: Vec<String>,
}

/// Where `file:line:col` puts a reader, leaving off what is unknown.
fn place_of(file: &Path, line: Option<usize>, column: Option<usize>) -> String {
    match (line, column) {
        (Some(line), Some(column)) => format!("{}:{line}:{column}", file.display()),
        (Some(line), None) => format!("{}:{line}", file.display()),
        _ => file.display().to_string(),
    }
}

fn hint_texts(diagnostic: &typst_harvest::Diagnostic) -> Vec<String> {
    diagnostic
        .hints
        .iter()
        .map(|hint| hint.message.clone())
        .collect()
}

impl LoadWarning {
    /// The warnings `diagnostics` hold for the file at `file`, whose evaluated
    /// main file is `main` in Typst's path form (root-relative, `/`-separated).
    pub(super) fn from_diagnostics(
        file: &Path,
        main: &str,
        diagnostics: &[typst_harvest::Diagnostic],
    ) -> Vec<Self> {
        diagnostics
            .iter()
            .map(|diagnostic| {
                let at = diagnostic.first_in(main);
                Self {
                    file: file.to_path_buf(),
                    line: at.map(|at| at.line),
                    column: at.map(|at| at.column),
                    message: diagnostic.message.clone(),
                    hints: hint_texts(diagnostic),
                }
            })
            .collect()
    }

    /// Where the warning is, as a reader opens it: `file:line:col`, or less
    /// when Typst gave less.
    #[must_use]
    pub fn place(&self) -> String {
        place_of(&self.file, self.line, self.column)
    }
}

impl LoadError {
    /// The failure of the file at `file`, whose text is `source`, to harvest.
    ///
    /// The place is the first point of an eval diagnostic inside the file
    /// itself, its trace walked when Typst raised it in the prelude. Typst
    /// leaves a show-everything rule out of that trace, so an error raised
    /// elsewhere that reaches no line of the file falls back to the line of
    /// the file's `task.with(…)` call, which applied the rule, and lies in
    /// that call. The hints are the ones on the diagnostic that gave the
    /// place, else on the first.
    #[must_use]
    pub(crate) fn harvest(file: PathBuf, err: &HarvestError, source: &str) -> Self {
        let message = err.to_string();
        let error = match err {
            HarvestError::Eval { error, .. } => error,
            HarvestError::File(_) | HarvestError::World(_) => {
                return Self {
                    file,
                    message,
                    ..Self::default()
                };
            }
        };
        let spans = TaskCallSpans::parse(source);
        let found = error
            .diagnostics
            .iter()
            .find_map(|diag| diag.first_in(&error.main).map(|at| (diag, at)));
        let hints = found
            .map(|(diag, _)| diag)
            .or(error.diagnostics.first())
            .map(hint_texts)
            .unwrap_or_default();
        let (line, column, region) = if let Some((_, at)) = found {
            let region = spans.region_at(source, at.line, at.column);
            (Some(at.line), Some(at.column), region)
        } else {
            let raised_elsewhere = error.diagnostics.iter().any(|diag| diag.location.is_some());
            match raised_elsewhere.then(|| spans.call_line()).flatten() {
                Some(call) => (Some(call), None, CallRegion::Call),
                None => (None, None, CallRegion::Outside),
            }
        };
        Self {
            file,
            line,
            column,
            message,
            hints,
            region: Box::new(region),
        }
    }

    /// Where the failure is, as a reader opens it: `file:line:col`, or less
    /// when no column or line is known.
    #[must_use]
    pub fn place(&self) -> String {
        place_of(&self.file, self.line, self.column)
    }
}
