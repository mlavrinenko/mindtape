//! A [`Diag`] as a JSON object, for a command that reports its problems on
//! stdout (`mt check --json`).

use serde_json::{Value, json};

use super::{Diag, render};

impl Diag {
    /// The problem as one object: `severity` (`error` or `warning`), `file`
    /// and `line` (`null` when it has none), `place` (the text the plain
    /// report starts with), `rule`, `message` (without place or rule) and
    /// `hints`.
    pub(crate) fn to_json(&self) -> Value {
        let (file, line) = self
            .0
            .site
            .as_ref()
            .map_or((None, None), |(file, line)| (Some(file.as_str()), *line));
        json!({
            "severity": self.0.kind.label(),
            "file": file,
            "line": line,
            "place": self.0.place,
            "rule": self.0.rule,
            "message": render::text(self),
            "hints": self.0.hints.iter().map(ToString::to_string).collect::<Vec<_>>(),
        })
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
                "message": "bad title", "hints": [],
            })
        );
    }

    #[test]
    fn a_warning_with_no_place_has_null_file_and_line() {
        assert_eq!(
            Diag::warning("careful").to_json(),
            json!({
                "severity": "warning", "file": null, "line": null, "place": null,
                "rule": null, "message": "careful", "hints": [],
            })
        );
    }
}
