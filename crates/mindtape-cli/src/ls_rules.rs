//! `[[ls.rules]]` cascade resolution for bare `mt ls`
//! (`truth/tasks/conditional-ls-rules.typ`).
//!
//! [`resolve`] is the single source both `ls.rs` (one-shot and
//! `--interactive`) and [`crate::util`]'s handle-ordinal resolution call, so
//! `mt ls`'s printed list and `mt go`'s ordinals never disagree: an explicit
//! CLI filter or `@view` is used verbatim (rules and the default view are
//! skipped); otherwise the first `[[ls.rules]]` entry whose `condition` holds
//! (or is absent) composes its `import-views` left-to-right (per-facet
//! last-wins) then applies its own manual overrides; otherwise the project's
//! default view; otherwise match-all. [`resolve_bare`] is the same cascade
//! without the CLI-argument layer, for callers (`util::default_view`) that
//! only ever want the bare-`mt ls` view.

use std::io::IsTerminal;

use anyhow::{Result, anyhow};
use mindtape::{Condition, ConditionEnv, Config, Facet, LsRule, ViewSpec, parse_import_view};

use crate::ls::LsArgs;
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

/// Source attribution for [`Bundle`], driving the human trailer and the
/// `mt ls --json` envelope. Each facet's `_from` is a view name, a rule's
/// [`rule_label`], `"manual"` (a literal CLI argument or rule override won),
/// or `"match-all"` (nothing applied — the built-in default).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Applied {
    pub rule: Option<String>,
    pub filter_from: String,
    pub filter_value: String,
    pub sort_from: String,
    pub sort_value: Vec<String>,
    pub columns_from: String,
    pub columns_value: Vec<String>,
}

/// The source label meaning "nothing applied here — the built-in default".
const MATCH_ALL: &str = "match-all";
/// The source label for a literal CLI argument or a rule's own manual override.
const MANUAL: &str = "manual";

impl Applied {
    fn match_all() -> Self {
        Self {
            filter_from: MATCH_ALL.to_owned(),
            sort_from: MATCH_ALL.to_owned(),
            columns_from: MATCH_ALL.to_owned(),
            ..Self::default()
        }
    }

    /// Whether nothing worth mentioning applied: no rule fired and every
    /// facet is still at the built-in default. The trailer stays silent for
    /// this case; the JSON envelope still reports it (`"rule": null`, every
    /// facet `"from": "match-all"`).
    #[must_use]
    pub fn is_trivial(&self) -> bool {
        self.rule.is_none()
            && self.filter_from == MATCH_ALL
            && self.sort_from == MATCH_ALL
            && self.columns_from == MATCH_ALL
    }
}

/// Build the environment [`mindtape::Condition`]s evaluate against: the
/// current git branch (from `project.root`'s `.git`), today's date, a full
/// snapshot of the process environment, whether stdout is a terminal, and the
/// active output format.
///
/// This is the one place the CLI reads those ambient facts —
/// [`mindtape::condition`] is pure by design and takes them injected. The same
/// environment drives both `[[ls.rules]]` and the per-column
/// `[ls.columns.<name>] when`, so a condition means the same thing wherever it
/// is written.
///
/// `format` is `""` for a caller with no active format — handle resolution
/// ([`crate::util::default_view`]) resolves a view to address a task, not to
/// render one — and `""` compares unequal to every real format name.
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
/// layer — this is what [`crate::util`]'s handle resolution wants, and what
/// [`resolve`] falls back to when the caller passed no filter.
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

/// Resolve the full `mt ls` listing: an explicit CLI filter or `@view` wins
/// outright (rules and the default view are skipped); otherwise
/// [`resolve_bare`]; either way, an explicit `--sort` / `--columns` flag has
/// the final word.
///
/// # Errors
/// Returns an error if an explicit `@view` / `--sort @view` / `--columns
/// @view` names an undeclared view, or (see [`resolve_bare`]).
pub fn resolve(
    project: &util::Project,
    args: &LsArgs,
    env: &ConditionEnv,
) -> Result<(Bundle, Applied)> {
    let (bundle, applied) = match &args.filter {
        Some(expr) => explicit_bundle(&project.config, expr)?,
        None => resolve_bare(project, env)?,
    };
    apply_cli_overrides(project, args, bundle, applied)
}

/// The bundle for an explicit CLI filter argument: a bare `@name` draws the
/// named view's whole bundle (attributed to it by name); anything else
/// (including the empty, match-all filter) is a manual filter with no
/// sort/columns contribution.
fn explicit_bundle(config: &Config, expr: &str) -> Result<(Bundle, Applied)> {
    if let Some(name) = bare_view_ref(expr) {
        let view = config
            .views
            .get(name)
            .ok_or_else(|| util::unknown_view_error(name, &config.views))?;
        return Ok(view_bundle(name, view));
    }
    let bundle = Bundle {
        filter: expr.to_owned(),
        sort: Vec::new(),
        columns: Vec::new(),
    };
    let applied = Applied {
        filter_from: MANUAL.to_owned(),
        filter_value: expr.to_owned(),
        ..Applied::match_all()
    };
    Ok((bundle, applied))
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
        rule: None,
        filter_from: name.to_owned(),
        filter_value: bundle.filter.clone(),
        sort_from: source_if_nonempty(name, &bundle.sort),
        sort_value: bundle.sort.clone(),
        columns_from: source_if_nonempty(name, &bundle.columns),
        columns_value: bundle.columns.clone(),
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
        .map_err(|err| anyhow!("ls.rules condition {expr:?}: {err}"))
}

/// A rule's display identity: its `name`, else its `condition` text, else
/// empty (an unnamed, unconditional catch-all rule).
fn rule_label(rule: &LsRule) -> String {
    rule.name
        .clone()
        .or_else(|| rule.condition.clone())
        .unwrap_or_default()
}

/// Compose a matched rule's effective bundle: `import-views` left-to-right
/// (per-facet last-wins), then the rule's own manual overrides (highest
/// precedence). The default view is not a base layer beneath a matched rule —
/// only what the rule itself imports or overrides applies.
fn compose_rule(config: &Config, rule: &LsRule) -> Result<(Bundle, Applied)> {
    let label = rule_label(rule);
    let mut applied = Applied {
        rule: Some(label),
        ..Applied::match_all()
    };
    for entry in &rule.import_views {
        let parsed = parse_import_view(entry).map_err(|msg| anyhow!("{msg}"))?;
        let view = config
            .views
            .get(&parsed.view)
            .ok_or_else(|| util::unknown_view_error(&parsed.view, &config.views))?;
        apply_import_entry(&mut applied, &parsed.facets, &parsed.view, view);
    }
    if let Some(filter) = &rule.filter {
        applied.filter_from = MANUAL.to_owned();
        applied.filter_value = filter.clone();
    }
    if !rule.sort.is_empty() {
        applied.sort_from = MANUAL.to_owned();
        applied.sort_value = splice_view_refs(&rule.sort, config, |view| &view.sort)?;
    }
    if !rule.columns.is_empty() {
        applied.columns_from = MANUAL.to_owned();
        applied.columns_value = splice_view_refs(&rule.columns, config, |view| &view.columns)?;
    }
    let bundle = Bundle {
        filter: applied.filter_value.clone(),
        sort: applied.sort_value.clone(),
        columns: applied.columns_value.clone(),
    };
    Ok((bundle, applied))
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
            return Err(anyhow!(
                "unknown column `{name}`, expected one of {}{}",
                mindtape::LS_COLUMNS.join(", "),
                mindtape::suggest::suggestion(name, &mindtape::LS_COLUMNS)
            ));
        }
    }
    Ok(names)
}

/// The human trailer line, grouped by winning source in first-appearance
/// order (filter, then sort, then columns); `None` when [`Applied::is_trivial`].
#[must_use]
pub fn trailer(applied: &Applied) -> Option<String> {
    if applied.is_trivial() {
        return None;
    }
    let facets = [
        ("filter", applied.filter_from.as_str()),
        ("sort", applied.sort_from.as_str()),
        ("columns", applied.columns_from.as_str()),
    ];
    let mut grouped: Vec<(&str, Vec<&str>)> = Vec::new();
    for (facet, from) in facets {
        if from == MATCH_ALL {
            continue;
        }
        if let Some(entry) = grouped.iter_mut().find(|(source, _)| *source == from) {
            entry.1.push(facet);
        } else {
            grouped.push((from, vec![facet]));
        }
    }
    if grouped.is_empty() {
        return None;
    }
    let parts: Vec<String> = grouped
        .into_iter()
        .map(|(source, facets)| format!("{source} ({})", facets.join(", ")))
        .collect();
    Some(format!("applied: {}", parts.join(" → ")))
}

#[cfg(test)]
#[path = "ls_rules_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "ls_rules_tests_applied.rs"]
mod tests_applied;
