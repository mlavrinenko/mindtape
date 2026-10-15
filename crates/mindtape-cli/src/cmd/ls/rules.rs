//! `[[ls.rules]]` cascade resolution for bare `mt ls`
//! (`truth/tasks/conditional-ls-rules.typ`).
//!
//! [`resolve`] is the single source `cmd/ls/mod.rs` (one-shot and `--interactive`)
//! calls. The cascade
//! runs bottom-up, per facet: the first `[[ls.rules]]` entry whose `condition`
//! holds (or is absent) composes its `import-views` left-to-right (per-facet
//! last-wins) then applies its own manual overrides, else the project's default
//! view, else match-all; then an explicit CLI filter argument overrides the
//! filter facet (a bare `@view` overrides every facet the view sets); then
//! `--sort` / `--columns` have the final word on theirs.
//!
//! Every layer is per facet, so narrowing a listing keeps the presentation the
//! layer beneath chose — filter answers "which tasks", sort and columns answer
//! "how do I read them"
//! (`truth/tasks/ls-rules-lose-their-sort-and-columns-facets-under-an-explicit-filter.typ`).
//! [`resolve_bare`] is the cascade without the CLI-argument layers.

use std::io::IsTerminal;

use crate::diag::Result;
use mindtape::{Condition, ConditionEnv, Config, Facet, LsRule, ViewSpec, parse_import_view};

pub use crate::cmd::ls::rules_applied::{Applied, footer};
use crate::cmd::ls::rules_applied::{MANUAL, MATCH_ALL};

use crate::cmd::ls::LsArgs;
use crate::diag::{Diag, Hint, Kind};
use crate::util;

/// A fully composed listing bundle: the three `mt ls` facets as plain owned
/// values. `filter` is always meaningful (`""` is the match-all filter, same
/// convention as [`mindtape::Filter::parse`]); an empty `sort` / `columns`
/// means "no override — keep the built-in default".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bundle {
    pub filter: String,
    pub sort: Vec<String>,
    pub columns: Vec<String>,
}

/// Build the environment [`mindtape::Condition`]s evaluate against: the
/// current git branch (from `project.root`'s `.git`), today's date, a full
/// snapshot of the process environment, whether stdout is a terminal, and the
/// active output format.
///
/// This is the one place the CLI reads those ambient facts —
/// [`mindtape::filter::condition`] is pure by design and takes them injected. The same
/// environment drives both `[[ls.rules]]` and the per-column
/// `[ls.columns.<name>] when`, so a condition means the same thing wherever it
/// is written.
///
/// `format` is the active output format's name; `""` compares unequal to every
/// real format name.
///
/// The listing facts ([`mindtape::ColumnFacts`]) stay at their all-false
/// default here and are filled in per column by
/// [`crate::render::columns::resolve_order`], the only place that knows them.
/// That is also what keeps `named()` from closing a loop: an `[[ls.rules]]`
/// condition — which may itself supply the `columns` override `named()` later
/// reads — sees the default, never a live answer.
pub(crate) fn build_env(project: &util::Project, format: &str) -> ConditionEnv {
    ConditionEnv {
        branch: mindtape::current_branch(&project.root),
        today: mindtape::today_iso(),
        env: std::env::vars().collect(),
        tty: std::io::stdout().is_terminal(),
        format: format.to_owned(),
        column: mindtape::ColumnFacts::default(),
    }
}

/// Resolve the bare-`mt ls` cascade only: the first matching `[[ls.rules]]`
/// entry, else the project's default view, else match-all. No CLI-argument
/// layer — this is what [`resolve`] falls back to when the caller passed no
/// filter.
///
/// # Errors
/// Returns an error if a rule's condition or a referenced view fails to
/// resolve (both already validated at config load, so this only recurs for a
/// hand-built, unvalidated config).
pub fn resolve_bare(project: &util::Project, env: &ConditionEnv) -> Result<(Bundle, Applied)> {
    if let Some(hit) = match_rule(project, env)? {
        return Ok(hit);
    }
    Ok(default_view_bundle(&project.config))
}

/// Resolve the full `mt ls` listing: [`resolve_bare`], then the explicit CLI
/// filter argument over the facets it speaks for, then `--sort` / `--columns`
/// over theirs.
///
/// # Errors
/// Returns an error if an explicit `@view` / `--sort @view` / `--columns
/// @view` names an undeclared view, or (see [`resolve_bare`]).
pub fn resolve(
    project: &util::Project,
    args: &LsArgs,
    env: &ConditionEnv,
) -> Result<(Bundle, Applied)> {
    let (bundle, applied) = resolve_bare(project, env)?;
    let rule_sources = applied.sources();
    let (bundle, applied) =
        apply_explicit_filter(&project.config, args.filter.as_deref(), bundle, applied)?;
    let (bundle, mut applied) = apply_cli_overrides(project, args, bundle, applied)?;
    applied.note_supersession(&rule_sources);
    Ok((bundle, applied))
}

/// Layer the explicit CLI filter argument over what the cascade resolved. A
/// bare `@name` is the caller naming a view, so it overrides every facet that
/// view sets; anything else (including the empty, match-all filter) narrows the
/// rows and nothing more, leaving the sort and columns beneath it standing.
fn apply_explicit_filter(
    config: &Config,
    filter: Option<&str>,
    bundle: Bundle,
    mut applied: Applied,
) -> Result<(Bundle, Applied)> {
    let Some(expr) = filter else {
        return Ok((bundle, applied));
    };
    if let Some(name) = bare_view_ref(expr) {
        let view = config
            .views
            .get(name)
            .ok_or_else(|| util::unknown_view_error(name, &config.views))?;
        apply_import_entry(&mut applied, &Facet::ALL, name, view);
    } else {
        applied.filter_from = MANUAL.to_owned();
        applied.filter_value = expr.to_owned();
    }
    Ok((bundle_of(&applied), applied))
}

/// The effective bundle an [`Applied`] describes — its three `_value` facets,
/// which are the resolved values themselves, not just attribution.
fn bundle_of(applied: &Applied) -> Bundle {
    Bundle {
        filter: applied.filter_value.clone(),
        sort: applied.sort_value.clone(),
        columns: applied.columns_value.clone(),
    }
}

/// A filter argument that is exactly one `@name` token (bundle mode), yielding
/// the bare name; `None` for a compound expression.
fn bare_view_ref(filter: &str) -> Option<&str> {
    let name = filter.trim().strip_prefix('@')?;
    let ok = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    ok.then_some(name)
}

/// A named view's bundle, attributed to it wholesale: the filter facet always
/// (an empty filter is still a meaningful match-all declaration), sort and
/// columns only when the view actually sets them.
fn view_bundle(name: &str, view: &ViewSpec) -> (Bundle, Applied) {
    let bundle = Bundle {
        filter: view.filter.clone(),
        sort: view.sort.clone(),
        columns: view.columns.clone(),
    };
    let applied = Applied {
        filter_from: name.to_owned(),
        filter_value: bundle.filter.clone(),
        sort_from: source_if_nonempty(name, &bundle.sort),
        sort_value: bundle.sort.clone(),
        columns_from: source_if_nonempty(name, &bundle.columns),
        columns_value: bundle.columns.clone(),
        ..Applied::default()
    };
    (bundle, applied)
}

fn source_if_nonempty(name: &str, values: &[String]) -> String {
    if values.is_empty() {
        MATCH_ALL.to_owned()
    } else {
        name.to_owned()
    }
}

/// The project's default-view bundle ([`Config::ls_default_view`]), or pure
/// match-all when none resolves.
fn default_view_bundle(config: &Config) -> (Bundle, Applied) {
    let Some(view) = config.ls_default_view() else {
        return (Bundle::default(), Applied::match_all());
    };
    let name = config.ls.default_view.as_deref().unwrap_or("default");
    view_bundle(name, view)
}

/// Evaluate `[[ls.rules]]` top to bottom; the first entry whose condition
/// holds (or is absent) wins.
fn match_rule(project: &util::Project, env: &ConditionEnv) -> Result<Option<(Bundle, Applied)>> {
    let config = &project.config;
    if config.ls.rules.is_empty() {
        return Ok(None);
    }
    for rule in &config.ls.rules {
        if condition_holds(rule, env)? {
            return Ok(Some(compose_rule(config, rule)?));
        }
    }
    Ok(None)
}

fn condition_holds(rule: &LsRule, env: &ConditionEnv) -> Result<bool> {
    let Some(expr) = &rule.condition else {
        return Ok(true);
    };
    Condition::parse(expr)
        .and_then(|condition| condition.matches(env))
        .map_err(|err| Diag::from(err).context("in ls.rules condition", expr))
}

/// A rule's display identity: its `name`, else its `condition` text, else
/// [`UNNAMED_RULE`]. Never empty — the label is a source name in its own right
/// now (a rule's own overrides are attributed to it), and an empty one would
/// render as `applied:  (filter)`.
fn rule_label(rule: &LsRule) -> String {
    rule.name
        .clone()
        .or_else(|| rule.condition.clone())
        .unwrap_or_else(|| UNNAMED_RULE.to_owned())
}

/// The label for an unnamed, unconditional catch-all rule — the one rule shape
/// with neither a `name` nor a `condition` to be known by.
const UNNAMED_RULE: &str = "rule";

/// Compose a matched rule's effective bundle: `import-views` left-to-right
/// (per-facet last-wins), then the rule's own manual overrides (highest
/// precedence within the rule). The default view is not a base layer beneath a
/// matched rule — only what the rule itself imports or overrides applies.
///
/// A rule's own overrides are attributed to the rule, not to `manual`: an
/// explicit CLI argument can now win one facet while the rule keeps another,
/// and one shared label would report the two as one source.
fn compose_rule(config: &Config, rule: &LsRule) -> Result<(Bundle, Applied)> {
    let label = rule_label(rule);
    let mut applied = Applied {
        rule: Some(label.clone()),
        ..Applied::match_all()
    };
    for entry in &rule.import_views {
        let parsed = parse_import_view(entry).map_err(|msg| Diag::verbatim(Kind::Failed, msg))?;
        let view = config
            .views
            .get(&parsed.view)
            .ok_or_else(|| util::unknown_view_error(&parsed.view, &config.views))?;
        apply_import_entry(&mut applied, &parsed.facets, &parsed.view, view);
    }
    if let Some(filter) = &rule.filter {
        applied.filter_from = label.clone();
        applied.filter_value = filter.clone();
    }
    if !rule.sort.is_empty() {
        applied.sort_from = label.clone();
        applied.sort_value = splice_view_refs(&rule.sort, config, |view| &view.sort)?;
    }
    if !rule.columns.is_empty() {
        applied.columns_from = label;
        applied.columns_value = splice_view_refs(&rule.columns, config, |view| &view.columns)?;
    }
    Ok((bundle_of(&applied), applied))
}

/// Fold one `import-views` entry into the accumulating `applied`, for each
/// facet it selects. Filter always overwrites (even an empty filter is a
/// meaningful match-all declaration); sort/columns only overwrite when the
/// view actually sets them, so a later entry that omits a facet cannot erase
/// an earlier one's contribution.
fn apply_import_entry(applied: &mut Applied, facets: &[Facet], view_name: &str, view: &ViewSpec) {
    for facet in facets {
        match facet {
            Facet::Filter => {
                applied.filter_from = view_name.to_owned();
                applied.filter_value = view.filter.clone();
            }
            Facet::Sort if !view.sort.is_empty() => {
                applied.sort_from = view_name.to_owned();
                applied.sort_value = view.sort.clone();
            }
            Facet::Columns if !view.columns.is_empty() => {
                applied.columns_from = view_name.to_owned();
                applied.columns_value = view.columns.clone();
            }
            Facet::Sort | Facet::Columns => {}
        }
    }
}

/// Splice each `@view` entry in `specs` with `field(view)`; a literal entry
/// passes through unchanged. Shared by a rule's manual `sort` / `columns`
/// override (here) and the CLI `--sort` flag ([`apply_cli_overrides`]) — same
/// splice semantics, different source field.
fn splice_view_refs(
    specs: &[String],
    config: &Config,
    field: impl Fn(&ViewSpec) -> &[String],
) -> Result<Vec<String>> {
    let mut out = Vec::with_capacity(specs.len());
    for spec in specs {
        if let Some(name) = spec.strip_prefix('@') {
            let view = config
                .views
                .get(name)
                .ok_or_else(|| util::unknown_view_error(name, &config.views))?;
            out.extend(field(view).iter().cloned());
        } else {
            out.push(spec.clone());
        }
    }
    Ok(out)
}

/// Layer the CLI's own `--sort` / `--columns` flags on top of `(bundle,
/// applied)`, highest precedence. Mirrors the pre-B2 `effective_sort` /
/// `resolve_columns` behaviour exactly, now also updating the attribution.
fn apply_cli_overrides(
    project: &util::Project,
    args: &LsArgs,
    mut bundle: Bundle,
    mut applied: Applied,
) -> Result<(Bundle, Applied)> {
    let config = &project.config;
    if !args.flags.sort.is_empty() {
        bundle.sort = splice_view_refs(&args.flags.sort, config, |view| &view.sort)?;
        applied.sort_from = MANUAL.to_owned();
        applied.sort_value = bundle.sort.clone();
    }
    if let Some(spec) = &args.flags.columns {
        bundle.columns = resolve_columns(spec, config)?;
        applied.columns_from = MANUAL.to_owned();
        applied.columns_value = bundle.columns.clone();
    }
    Ok((bundle, applied))
}

/// The hint for a column name that is nothing like a real one.
const COLUMN_NAMES: &str = "columns are slug#, status, ready, priority, due, assignee, title, why, children, progress, file";

/// Resolve a `--columns` spec (a comma list of column names, or `@view`) to a
/// validated list of `[ls]` column names.
fn resolve_columns(spec: &str, config: &Config) -> Result<Vec<String>> {
    let names: Vec<String> = if let Some(name) = spec.strip_prefix('@') {
        let view = config
            .views
            .get(name)
            .ok_or_else(|| util::unknown_view_error(name, &config.views))?;
        view.columns.clone()
    } else {
        spec.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    };
    for name in &names {
        if !mindtape::LS_COLUMNS.contains(&name.as_str()) {
            let diag = Diag::usage("unknown column").value(name);
            return Err(diag.did_you_mean_or(name, &mindtape::LS_COLUMNS, Hint::new(COLUMN_NAMES)));
        }
    }
    Ok(names)
}

#[cfg(test)]
#[path = "rules_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "rules_tests_mutants.rs"]
mod tests_mutants;
