//! `body:` guarantee: synthesize a minimal `== Summary` when a record's body
//! is empty and the project's body-header rules would otherwise fail every
//! import outright — the policy home for "every imported file passes
//! `mt check`" (`emit.rs` stays pure and never synthesizes on its own).

use crate::canonical::CanonicalTask;
use crate::config::{Config, HeaderAction, HeaderPathSegment};

use super::report::ReportEntry;

/// The record's body, trimmed, or a synthesized `== Summary` from the title
/// when empty and the project requires one. Detection is best-effort: it
/// looks for a `require`/`unique` header rule whose single-segment path names
/// (or, for a regex segment, textually mentions) "summary" — precise enough
/// for the common single-rule case this project itself declares
/// (`.mindtape/config.toml`'s `path = ["Summary"], action = "unique"`)
/// without pulling in a regex engine just to decide whether to synthesize.
pub(crate) fn map_body(
    canonical: &CanonicalTask,
    config: &Config,
    out: &mut Vec<ReportEntry>,
) -> String {
    let body = canonical.body.trim();
    if !body.is_empty() {
        return body.to_owned();
    }
    if !requires_summary_heading(config) {
        return String::new();
    }
    out.push(ReportEntry::defaulted(
        "body",
        "empty body under a Summary-requiring project; synthesized `== Summary`".to_owned(),
    ));
    format!("== Summary\n\n{}", canonical.title)
}

fn requires_summary_heading(config: &Config) -> bool {
    config.tasks.body.headers.rules.iter().any(|rule| {
        matches!(rule.action, HeaderAction::Require | HeaderAction::Unique)
            && matches!(rule.path.as_slice(), [only] if segment_names_summary(only))
    })
}

fn segment_names_summary(segment: &HeaderPathSegment) -> bool {
    match segment {
        HeaderPathSegment::Exact(text) => text.eq_ignore_ascii_case("summary"),
        HeaderPathSegment::Pattern { pattern, .. } => {
            pattern.to_ascii_lowercase().contains("summary")
        }
    }
}
