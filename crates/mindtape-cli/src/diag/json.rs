//! A [`Diag`] as a JSON object, for a command that reports its problems on
//! stdout (`mt check --json`).

use serde::Serialize;
#[cfg(test)]
use serde_json::Value;

use super::{Diag, Place, render};

/// One problem as `mt check --json` prints it.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ProblemJson {
    /// `error` or `warning`.
    pub severity: String,
    /// The file the problem sits in, `/`-separated, or `null`.
    pub file: Option<String>,
    /// The line in that file, or `null`.
    pub line: Option<usize>,
    /// The text the plain report starts with, `/`-separated like `file`.
    pub place: Option<String>,
    /// The rule that refused the task, or `null`.
    pub rule: Option<String>,
    /// The problem's message, without its place or rule.
    pub message: String,
    /// The next steps the report names.
    pub hints: Vec<String>,
    /// The context the report adds.
    pub notes: Vec<String>,
}

impl From<&Diag> for ProblemJson {
    fn from(diag: &Diag) -> Self {
        let (file, line) = match &diag.0.place {
            Some(Place::File { path, line, .. }) => (Some(mindtape::slash_path(path)), *line),
            _ => (None, None),
        };
        Self {
            severity: diag.0.kind.label().to_owned(),
            file,
            line,
            place: diag
                .0
                .place
                .as_ref()
                .map(|place| place.text(mindtape::slash_path)),
            rule: diag.0.rule.clone(),
            message: render::text(diag),
            hints: diag.0.hints.iter().map(ToString::to_string).collect(),
            notes: diag.0.notes.iter().map(ToString::to_string).collect(),
        }
    }
}

impl Diag {
    /// The problem as one object: `severity` (`error` or `warning`), `file`
    /// and `line` (`null` when it has none), `place` (the text the plain
    /// report starts with, `/`-separated like `file`), `rule`, `message` (without place or rule), `hints`
    /// and `notes`.
    #[cfg(test)]
    pub(crate) fn to_json(&self) -> Value {
        serde_json::to_value(ProblemJson::from(self)).expect("a problem serialises to a value")
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use crate::diag::Diag;

    #[test]
    fn a_placed_breach_names_its_file_line_rule_and_message() {
        let diag = Diag::refused("")
            .at(Path::new("tasks/a.typ"), 7, None)
            .rule("rules.x")
            .cause(&std::io::Error::other("bad title"));

        assert_eq!(
            diag.to_json(),
            json!({
                "severity": "error", "file": "tasks/a.typ", "line": 7,
                "place": "tasks/a.typ:7", "rule": "rules.x",
                "message": "bad title", "hints": [], "notes": [],
            })
        );
    }

    #[test]
    fn a_warning_with_no_place_has_null_file_and_line() {
        assert_eq!(
            Diag::warning("careful").to_json(),
            json!({
                "severity": "warning", "file": null, "line": null, "place": null,
                "rule": null, "message": "careful", "hints": [], "notes": [],
            })
        );
    }
}
