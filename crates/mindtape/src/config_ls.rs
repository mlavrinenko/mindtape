//! `[ls]` config: `mt ls` rendering defaults.
//!
//! Filter / sort defaults, the global date format, and the column render
//! order. The column-name vocabulary ([`LS_COLUMNS`]) lives here as the single
//! validated source shared by `[ls.columns]`, `[ls] columns-order`, and any
//! consumer renderer; one column's own settings (`visibility`, conditional
//! `rules`) live in [`crate::config_ls_columns`].

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};

use crate::config_ls_columns::{ColumnConfig, unknown_column};
use crate::config_views::ViewSpec;
use crate::sort::validate_sort_spec;

/// Canonical `mt ls` column names, in built-in render order. The single
/// validated vocabulary shared by `[ls.columns]` and `[ls] columns-order` (and
/// any consumer): `#` is the handle column, `title` the flex column, `file`
/// (last) the task's source path — opt-in only, since its `auto` visibility
/// means "was it named explicitly", not "does it carry text" (its cell is
/// never blank; see `crate::config_ls::Visibility`).
///
/// `assignee` joins the short scalars ahead of `title`, and reads `auto` as a
/// third thing again — "do the rows disagree". Neither existing rule fits: the
/// ordinary non-empty test would pin a column onto `mt ls @agent` repeating one
/// name down every row the filter already named, and `file`'s
/// named-explicitly test would keep it off a mixed listing, which is the one
/// place it says something.
pub const LS_COLUMNS: [&str; 8] = [
    "#", "status", "ready", "priority", "due", "assignee", "title", "file",
];

/// `[ls]` — `mt ls` defaults.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LsConfig {
    /// Name of the view that drives the default `mt ls` listing when the
    /// caller passes no filter: falls back to the `[views.default]` view when
    /// this is unset, or matches every task when neither is declared.
    /// Validated at config load.
    pub default_view: Option<String>,
    /// Default sort chain for `mt ls` when the caller passes no `--sort`.
    /// Same spec grammar as `--sort`: `"field"` or `"field:asc"` / `"field:desc"`.
    /// Validated at config load. When absent the shipped built-in chain applies.
    pub default_sort: Vec<String>,
    /// Global date display format applied to every rendered date column.
    /// Tokens (longest-match-first): `YYYY` 4-digit year, `YY` 2-digit year,
    /// `MMMM` full month name, `MMM` abbreviated month, `MM` zero-padded month,
    /// `M` bare month, `DD` zero-padded day, `D` bare day.
    /// Non-token characters pass through literally.
    /// Default `None` renders ISO `YYYY-MM-DD`.
    pub date_format: Option<String>,
    /// Per-column display settings, keyed by column name. Unlisted columns
    /// keep their default display.
    pub columns: BTreeMap<String, ColumnConfig>,
    /// Explicit column order and selection: when set, only the listed
    /// columns render, in that order. Empty (the default) keeps the built-in
    /// column set and order.
    pub columns_order: Vec<String>,
    /// Display settings (glyph, colour, label) for the ready and blocked
    /// states shown in the READY column. Defaults to `MindTape`'s built-in
    /// values, so leaving this unset keeps the current behaviour.
    pub ready: ReadyConfig,
    /// Colour for the overdue due-date accent. Defaults to `"red"`.
    pub overdue_color: Option<String>,
    /// Ordered auto-activation rules applied to a bare `mt ls` (no CLI
    /// filter, no `@view`): the first rule whose condition holds (or that
    /// has none) wins and supplies its filter, sort, and columns.
    pub rules: Vec<LsRule>,
    /// Hyperlink output for the `#` and `file` columns' cells: `auto`,
    /// `always`, or `never` (the default). Mirrors `--hyperlinks`, which wins
    /// over this key when passed — see [`HyperlinkChoice`] for what each
    /// value means and why the default is not `auto`.
    pub hyperlinks: HyperlinkChoice,
    /// Template for the shared per-row hyperlink target URI both `#` and
    /// `file` link to: `{abs}` (the absolute path, the project root joined
    /// with the task's `file`) and `{rel}` (the same working-directory-relative
    /// path the `file` column displays) are substituted. Defaults to
    /// `file://{abs}`; point it at another scheme (e.g. an editor's own URI
    /// handler) to open elsewhere.
    pub hyperlink_target: Option<String>,
    /// Default `contains()` haystack when a call names no field: an ordered
    /// list of field names drawn from `crate::fields::is_contains_field`'s
    /// vocabulary (every `crate::fields::FIELDS` name, plus the text-only
    /// tier `body`/`note`/`tags`). Empty (the default) keeps the shipped
    /// sweep — `title`, `id`, `body` — unchanged; a project states its own
    /// answer to widen it (e.g. to include `note`, the `links[].note` prose,
    /// or `tags`) or narrow it. An explicitly named field in a `contains()`
    /// call (`contains("x", note)`) always works regardless of this default.
    /// Validated at config load
    /// ([`deserialize_contains_fields`]): an unknown name is a config error
    /// naming the field and listing the valid set, never a silent drop.
    #[serde(deserialize_with = "deserialize_contains_fields")]
    pub contains_fields: Vec<String>,
}

/// Deserialize `[ls] contains-fields`, rejecting any name outside
/// `crate::fields::is_contains_field`'s vocabulary — the same one an explicit
/// `contains("x", field)` call validates against. Runs only when the key is
/// present (a missing key keeps the struct-level `#[serde(default)]` empty
/// vec, silently valid since nothing was configured); a bad name surfaces as
/// a config parse error at load, naming the field and the valid set, rather
/// than a silently ignored entry.
fn deserialize_contains_fields<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let names = Vec::<String>::deserialize(deserializer)?;
    for name in &names {
        if !crate::fields::is_contains_field(name) {
            return Err(serde::de::Error::custom(format!(
                "ls.contains-fields: unknown field `{name}`, expected one of {}",
                crate::fields::contains_field_names().join(", ")
            )));
        }
    }
    Ok(names)
}

/// `[ls] hyperlinks` — whether the `#` and `file` columns' cells carry an
/// OSC 8 hyperlink sequence (`ESC ] 8 ; ; <uri> ST <text> ESC ] 8 ; ; ST`) in
/// a standalone terminal, both pointing at the same per-row target. The `mt`
/// binary's `--hyperlinks` flag is this same choice, mirrored one-to-one in
/// the CLI crate (which alone depends on `clap`) as
/// `crate::render::link::HyperlinkChoice` there.
///
/// There is no terminal capability query for OSC 8 support, so `auto` is a
/// heuristic (stdout is a TTY and `TERM` is not `dumb`), never a detection —
/// and unlike colour, which degrades to nothing on a terminal that ignores
/// it, an unsupported terminal can render the OSC 8 payload as visible
/// garbage. That asymmetry is why the default is `never`, not `auto`
/// (`truth/tasks/hyperlink-the-file-column-with-osc-8.typ`).
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HyperlinkChoice {
    /// Enable hyperlinks only when stdout is a TTY and `TERM` is not `dumb`.
    Auto,
    /// Always emit OSC 8 hyperlink sequences.
    Always,
    /// Never emit hyperlink sequences.
    #[default]
    Never,
}

/// One `[[ls.rules]]` entry.
///
/// The first rule (top to bottom) whose `condition` holds — or that omits
/// `condition` entirely — drives the default `mt ls` listing. Its effective
/// facets are `import-views` composed left-to-right (per-facet last-wins),
/// then any manual `filter` / `sort` / `columns` override (highest
/// precedence). All fields are optional; a rule with none of them still
/// matches (by an absent `condition`) but contributes no facets.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LsRule {
    /// Shown in the trailer and the JSON envelope's `applied.rule`; falls
    /// back to the condition text when unset.
    pub name: Option<String>,
    /// Condition expression over environment predicates such as
    /// `on_branch`, `env`, `weekday`, and `now`. Absent means the rule always
    /// matches.
    pub condition: Option<String>,
    /// Ordered list of views to import facets from, composed left-to-right
    /// (later entries win on conflict). Each entry is a view name (`"view"`,
    /// importing its filter, sort, and columns) or a scoped form
    /// (`"view(filter, sort)"`) naming just the facets to import.
    pub import_views: Vec<String>,
    /// Manual filter override — highest precedence for the filter facet.
    /// Same grammar as `[views]`; may use `@view` / `in_view`.
    pub filter: Option<String>,
    /// Manual sort override — highest precedence for the sort facet. Same
    /// spec grammar as `--sort`; an entry may splice `@view`.
    pub sort: Vec<String>,
    /// Manual column list — highest precedence for the columns facet. An
    /// entry may reference a view's columns with `@view`.
    pub columns: Vec<String>,
}

/// Which facets an `import-views` entry contributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facet {
    /// The filter facet.
    Filter,
    /// The sort facet.
    Sort,
    /// The columns facet.
    Columns,
}

/// One resolved `import-views` entry: the view it names, plus which facets it
/// contributes. A bare `"view"` entry selects all three; `"view(sort,
/// columns)"` selects a subset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportView {
    /// The named view.
    pub view: String,
    /// Facets this entry contributes, in declaration order (may repeat; the
    /// caller composing facets treats that as harmless).
    pub facets: Vec<Facet>,
}

/// Parse one `import-views` entry: `"view"` (all facets) or `"view(facet,
/// ...)"` (a selector subset of `filter` / `sort` / `columns`).
///
/// # Errors
/// Returns a message for an empty entry, a missing view name, an unterminated
/// selector, an empty selector (`"view()"` — drop the parens for all facets
/// instead), or an unknown facet name.
pub fn parse_import_view(entry: &str) -> Result<ImportView, String> {
    let entry = entry.trim();
    if entry.is_empty() {
        return Err("empty import-views entry".to_owned());
    }
    let Some((view, rest)) = entry.split_once('(') else {
        return Ok(ImportView {
            view: entry.to_owned(),
            facets: vec![Facet::Filter, Facet::Sort, Facet::Columns],
        });
    };
    let view = view.trim();
    if view.is_empty() {
        return Err(format!(
            "malformed import-views entry `{entry}`: missing view name"
        ));
    }
    let Some(inner) = rest.strip_suffix(')') else {
        return Err(format!(
            "malformed import-views entry `{entry}`: missing closing `)`"
        ));
    };
    let inner = inner.trim();
    if inner.is_empty() {
        return Err(format!(
            "import-views entry `{entry}`: empty facet list; drop the parens for all facets"
        ));
    }
    let mut facets = Vec::new();
    for part in inner.split(',') {
        let part = part.trim();
        let facet = match part {
            "filter" => Facet::Filter,
            "sort" => Facet::Sort,
            "columns" => Facet::Columns,
            other => {
                return Err(format!(
                    "import-views entry `{entry}`: unknown facet `{other}`, expected filter, sort, or columns"
                ));
            }
        };
        facets.push(facet);
    }
    Ok(ImportView {
        view: view.to_owned(),
        facets,
    })
}

/// Validate every `[[ls.rules]]` entry: `condition` parses in the condition
/// grammar (which already rejects a task-field identifier or an unknown
/// predicate — see [`crate::condition::Condition::parse`]); each
/// `import-views` entry parses and names a declared view; a manual `filter`
/// parses and its `@view` / `in_view` references resolve; a manual `sort` /
/// `columns` entry is either a legal spec or an `@view` reference that
/// resolves. Returns one message per problem, prefixed `ls.rules[<i>]`.
///
/// A rule can only ever be a *sink* in the view-reference graph (nothing can
/// `import-views` or `@`-reference a rule back), so it cannot participate in
/// a cycle; [`crate::config_views::validate_views`] already guarantees the
/// views-only graph is acyclic, which is all the acyclicity this needs.
#[must_use]
pub fn validate_ls_rules(rules: &[LsRule], views: &BTreeMap<String, ViewSpec>) -> Vec<String> {
    let mut problems = Vec::new();
    for (index, rule) in rules.iter().enumerate() {
        let prefix = format!("ls.rules[{index}]");
        validate_condition(&prefix, rule, &mut problems);
        validate_import_views(&prefix, rule, views, &mut problems);
        validate_manual_filter(&prefix, rule, views, &mut problems);
        problems.extend(validate_view_spliced(
            &prefix,
            "sort",
            &rule.sort,
            views,
            validate_sort_spec,
        ));
        problems.extend(validate_view_spliced(
            &prefix,
            "columns",
            &rule.columns,
            views,
            |col| {
                if LS_COLUMNS.contains(&col) {
                    Ok(())
                } else {
                    Err(unknown_column(col))
                }
            },
        ));
    }
    problems
}

fn validate_condition(prefix: &str, rule: &LsRule, problems: &mut Vec<String>) {
    if let Some(condition) = &rule.condition
        && let Err(err) = crate::condition::Condition::parse(condition)
    {
        problems.push(format!("{prefix}.condition: {err}"));
    }
}

fn validate_import_views(
    prefix: &str,
    rule: &LsRule,
    views: &BTreeMap<String, ViewSpec>,
    problems: &mut Vec<String>,
) {
    for entry in &rule.import_views {
        match parse_import_view(entry) {
            Ok(parsed) if !views.contains_key(&parsed.view) => problems.push(format!(
                "{prefix}.import-views: unknown view `{}`{}",
                parsed.view,
                crate::suggest::hint("view", &parsed.view, views.keys())
            )),
            Ok(_) => {}
            Err(msg) => problems.push(format!("{prefix}.import-views: {msg}")),
        }
    }
}

fn validate_manual_filter(
    prefix: &str,
    rule: &LsRule,
    views: &BTreeMap<String, ViewSpec>,
    problems: &mut Vec<String>,
) {
    let Some(filter) = &rule.filter else {
        return;
    };
    match crate::filter::Filter::parse(filter) {
        Ok(parsed) => {
            for target in parsed.view_refs() {
                if !views.contains_key(&target) {
                    problems.push(format!(
                        "{prefix}.filter: unknown view reference @{target}{}",
                        crate::suggest::hint("view", &target, views.keys())
                    ));
                }
            }
        }
        Err(err) => problems.push(format!("{prefix}.filter: {err}")),
    }
}

/// Validate a manual `sort` / `columns` override chain: each entry is either
/// an `@view` splice (must resolve) or a literal spec checked by `check_spec`.
/// Returns one message per problem, prefixed `<prefix>.<facet>`.
fn validate_view_spliced(
    prefix: &str,
    facet: &str,
    entries: &[String],
    views: &BTreeMap<String, ViewSpec>,
    check_spec: impl Fn(&str) -> Result<(), String>,
) -> Vec<String> {
    let mut problems = Vec::new();
    for entry in entries {
        if let Some(name) = entry.strip_prefix('@') {
            if !views.contains_key(name) {
                problems.push(format!(
                    "{prefix}.{facet}: unknown view reference @{name}{}",
                    crate::suggest::hint("view", name, views.keys())
                ));
            }
        } else if let Err(msg) = check_spec(entry) {
            problems.push(format!("{prefix}.{facet}: {msg}"));
        }
    }
    problems
}

/// `[ls.ready]` — the READY column's per-state display (glyph, colour, label).
///
/// One block covering both `ready` and `blocked` states (rather than nesting
/// under `[ls.columns.ready]`, which only carries generic `visibility`):
/// `blocked` needs display attributes `ColumnConfig` has no room for. Every
/// field is optional so an omitted key falls back to the embedded floor's
/// value (or, absent even that, the renderer's own literal fallback).
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ReadyConfig {
    /// Glyph shown for a ready (unblocked, non-terminal) task.
    pub ready_glyph: Option<String>,
    /// Colour for the ready state: a named ANSI colour or `#RRGGBB`.
    pub ready_color: Option<String>,
    /// Display label for the ready state.
    pub ready_label: Option<String>,
    /// Glyph shown for a blocked task.
    pub blocked_glyph: Option<String>,
    /// Colour for the blocked state: a named ANSI colour or `#RRGGBB`.
    pub blocked_color: Option<String>,
    /// Display label for the blocked state.
    pub blocked_label: Option<String>,
}

#[cfg(test)]
#[path = "config_ls_tests.rs"]
mod tests;
