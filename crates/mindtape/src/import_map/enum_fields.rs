//! `kind:`/`tags:` policy mapping: enum membership and the tag allow-list.

use crate::config::Config;

use super::report::{MapOptions, ReportEntry};

/// Report when `kind` is set but the project declares a `kind` enum and the
/// value falls outside it. Never rewrites or drops `kind` — the mapped
/// [`crate::model::Task`] keeps whatever was authored; `mt check` re-flags it
/// too if the record still fails after import.
pub(crate) fn check_kind(kind: Option<&str>, config: &Config, out: &mut Vec<ReportEntry>) {
    let Some(field) = config.tasks.fields.get("kind") else {
        return;
    };
    let Some(value) = kind else {
        return;
    };
    if !field.values.is_empty() && !field.values.iter().any(|allowed| allowed == value) {
        out.push(ReportEntry::violated(
            "kind",
            format!("`{value}` is not an allowed kind"),
        ));
    }
}

/// Filter `tags` against `[tasks.tags] allowed`: an unlisted tag is kept and
/// reported as a violation by default, or dropped and reported when
/// [`MapOptions::drop_unmapped_tags`] is set. An empty allow-list leaves every
/// tag untouched (unconstrained, matching `validate.rs::check_tags`).
pub(crate) fn map_tags(
    tags: Vec<String>,
    config: &Config,
    opts: &MapOptions,
    out: &mut Vec<ReportEntry>,
) -> Vec<String> {
    let allowed = &config.tasks.tags.allowed;
    if allowed.is_empty() {
        return tags;
    }
    tags.into_iter()
        .filter_map(|tag| filter_tag(tag, allowed, opts, out))
        .collect()
}

fn filter_tag(
    tag: String,
    allowed: &[String],
    opts: &MapOptions,
    out: &mut Vec<ReportEntry>,
) -> Option<String> {
    if allowed.contains(&tag) {
        return Some(tag);
    }
    if opts.drop_unmapped_tags {
        out.push(ReportEntry::dropped(
            "tags",
            format!("`{tag}` is not an allowed tag; dropped"),
        ));
        None
    } else {
        out.push(ReportEntry::violated(
            "tags",
            format!("`{tag}` is not an allowed tag"),
        ));
        Some(tag)
    }
}
