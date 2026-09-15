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

#[cfg(test)]
mod tests {
    use super::super::report::ReportEntry;
    use crate::canonical::CanonicalTask;
    use crate::config::{
        Config, HeaderAction, HeaderPathSegment, HeaderRuleConfig, HeadersConfig, TaskBodyConfig,
        TasksConfig,
    };

    fn ex(text: &str) -> HeaderPathSegment {
        HeaderPathSegment::Exact(text.to_owned())
    }

    fn require_rule(path: Vec<HeaderPathSegment>) -> HeaderRuleConfig {
        HeaderRuleConfig {
            name: None,
            action: HeaderAction::Require,
            path,
            before: vec![],
            after: vec![],
            scope: vec![],
            allowed: vec![],
            when: None,
            message: None,
        }
    }

    fn forbid_rule(path: Vec<HeaderPathSegment>) -> HeaderRuleConfig {
        HeaderRuleConfig {
            name: None,
            action: HeaderAction::Forbid,
            path,
            before: vec![],
            after: vec![],
            scope: vec![],
            allowed: vec![],
            when: None,
            message: None,
        }
    }

    fn config_with_rules(rules: Vec<HeaderRuleConfig>) -> Config {
        Config {
            tasks: TasksConfig {
                body: TaskBodyConfig {
                    headers: HeadersConfig { rules },
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Runs `map_body` over a fresh "Fix Login" task under the given header
    /// `rules`, returning its result body and the report entries it pushed —
    /// the shared setup for the scenarios below.
    fn run_map_body(rules: Vec<HeaderRuleConfig>) -> (String, Vec<ReportEntry>) {
        let config = config_with_rules(rules);
        let task = CanonicalTask {
            title: "Fix Login".to_owned(),
            ..Default::default()
        };
        let mut out = Vec::new();
        let result = super::map_body(&task, &config, &mut out);
        (result, out)
    }

    #[test]
    fn matching_require_summary_synthesizes() {
        let (result, out) = run_map_body(vec![require_rule(vec![ex("Summary")])]);
        assert_eq!(result, "== Summary\n\nFix Login");
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn non_matching_action_does_not_synthesize() {
        let (result, out) = run_map_body(vec![forbid_rule(vec![ex("Summary")])]);
        assert_eq!(result, "");
        assert!(out.is_empty());
    }

    #[test]
    fn non_summary_segment_does_not_synthesize() {
        let (result, out) = run_map_body(vec![require_rule(vec![ex("NotSummary")])]);
        assert_eq!(result, "");
        assert!(out.is_empty());
    }
}
