//! The document `mt ls --json` prints inside its `$schema` envelope: the task
//! records plus how the listing was shaped, cut and matched.

use mindtape::{CanonicalTask, Config, Readiness, Task};
use serde::Serialize;

use crate::cmd::ls::rules::Applied;
use crate::render::output::Truncation;

/// The document-level facts `mt ls --json` wraps its records in: which rule
/// and view won each facet ([`Applied`]), what `--limit` cut
/// ([`Truncation`]), and the retired stems a `touches()` met. `mt eval --json` and the flip-hook table have neither
/// context and pass `None` to the JSON renderer, keeping the bare array.
#[derive(Debug, Clone, Copy)]
pub struct JsonContext<'a> {
    pub applied: &'a Applied,
    pub truncated: Option<Truncation>,
    /// Every stem a `touches()` call met that no task answers to, even
    /// through a rename ([`mindtape::config::Config::retired_refs`]).
    pub retired: &'a [String],
}

/// The document `$schema` envelope `mt ls --json` prints: the selected task
/// records and the context that shaped them.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct LsPayload {
    /// The selected task records.
    pub tasks: Vec<CanonicalTask>,
    /// Which rule and view won each facet.
    pub applied: AppliedJson,
    /// What `--limit` cut, or `null` for a complete listing.
    pub truncated: Option<TruncatedJson>,
    /// The stems a `touches()` call met that no task answers to.
    pub retired: Vec<String>,
}

/// `mt ls --json`'s `applied` field: the matching rule, whether it survived,
/// and the winning source of each facet.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct AppliedJson {
    /// The matching `[[ls.rules]]` label, or `null`.
    pub rule: Option<String>,
    /// Whether the rule matched and lost every facet.
    pub superseded: bool,
    /// The three facets' winning sources and values.
    pub facets: AppliedFacets,
}

/// The filter, sort and columns facets [`AppliedJson`] reports.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct AppliedFacets {
    /// Where the filter came from, and what it was.
    pub filter: AppliedFacet<String>,
    /// Where the sort came from, and what it was.
    pub sort: AppliedFacet<Vec<String>>,
    /// Where the columns came from, and what they were.
    pub columns: AppliedFacet<Vec<String>>,
}

/// One facet's winning source and effective value.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct AppliedFacet<T> {
    /// The view, rule label, `manual` or `match-all`.
    pub from: String,
    /// The value the listing used.
    pub value: T,
}

/// `mt ls --json`'s `truncated` field: what `--limit` cut.
#[derive(Debug, Clone, Copy, Serialize, schemars::JsonSchema)]
pub struct TruncatedJson {
    /// The rows this page showed.
    pub shown: usize,
    /// The rows the filter matched before the cut.
    pub total: usize,
    /// The `--skip` that continues from here.
    pub next_skip: usize,
}

impl AppliedJson {
    /// Project `applied` into its JSON form.
    fn new(applied: &Applied) -> Self {
        Self {
            rule: applied.rule.clone(),
            superseded: applied.rule_superseded,
            facets: AppliedFacets {
                filter: facet(&applied.filter_from, applied.filter_value.clone()),
                sort: facet(&applied.sort_from, applied.sort_value.clone()),
                columns: facet(&applied.columns_from, applied.columns_value.clone()),
            },
        }
    }
}

/// One facet from its source and effective value.
fn facet<T>(from: &str, value: T) -> AppliedFacet<T> {
    AppliedFacet {
        from: from.to_owned(),
        value,
    }
}

impl From<Truncation> for TruncatedJson {
    fn from(cut: Truncation) -> Self {
        Self {
            shown: cut.shown,
            total: cut.total,
            next_skip: cut.next_skip(),
        }
    }
}

/// Build the `mt ls --json` payload from the rows and the context that shaped
/// them.
pub(crate) fn ls_payload(
    tasks: &[Task],
    config: &Config,
    readiness: &Readiness,
    context: JsonContext<'_>,
) -> LsPayload {
    LsPayload {
        tasks: canonical_tasks(tasks, config, readiness),
        applied: AppliedJson::new(context.applied),
        truncated: context.truncated.map(TruncatedJson::from),
        retired: context.retired.to_vec(),
    }
}

/// Project every row into a [`CanonicalTask`], each carrying what it waits on.
pub(crate) fn canonical_tasks(
    tasks: &[Task],
    config: &Config,
    readiness: &Readiness,
) -> Vec<CanonicalTask> {
    tasks
        .iter()
        .map(|task| {
            let mut canonical = CanonicalTask::from_task(task, config);
            canonical.why = readiness.blockers(task).to_vec();
            canonical
        })
        .collect()
}
