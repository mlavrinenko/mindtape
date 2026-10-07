//! `[ls]` config: `mt ls` rendering defaults.
//!
//! Filter / sort defaults, the global date format, and the column render
//! order. The column-name vocabulary ([`LS_COLUMNS`]) lives here as the single
//! validated source shared by `[ls.columns]`, `[ls] columns-order`, and any
//! consumer renderer; one column's own settings (its `when` expression) live
//! in [`crate::config::ls_columns`].

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};

use crate::config::ls_columns::{ColumnConfig, unknown_column};
use crate::config::views::ViewSpec;
use crate::sort::validate_sort_spec;

/// Canonical `mt ls` column names, in built-in render order. The single
/// validated vocabulary shared by `[ls.columns]` and `[ls] columns-order` (and
/// any consumer): `slug#` is the stem hash column, `title` the flex column,
/// `file` (last) the task's source path — opt-in only, since its cell is never
/// blank and `nonempty()` would pin it permanently visible; the floor gives it
/// `named()` instead.
///
/// `assignee` joins the short scalars ahead of `title` on a third rule again —
/// `contested()`, "do the rows disagree". Neither of the others fits: the
/// ordinary `nonempty()` test would pin a column onto `mt ls @agent` repeating
/// one name down every row the filter already named, and `file`'s `named()`
/// would keep it off a mixed listing, which is the one place it says something.
///
/// All three live as one `when` expression per column in
/// `crates/mindtape/src/config/defaults.toml`, not in Rust — see
/// [`crate::config::ls_columns`].
pub const LS_COLUMNS: [&str; 11] = [
    "slug#", "status", "ready", "priority", "due", "assignee", "title", "why", "children",
    "progress", "file",
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
    /// Date format for every date column. Tokens, longest match first: `YYYY`
    /// 4-digit year, `YY` 2-digit year, `MMMM` full month name, `MMM`
    /// abbreviated month, `MM` zero-padded month, `M` bare month, `DD`
    /// zero-padded day, `D` bare day. Other characters print as written.
    /// Absent prints ISO `YYYY-MM-DD`.
    pub date_format: Option<String>,
    /// Per-column display settings, keyed by column name. The shipped
    /// defaults set `when` on every column; a column with no `when` never
    /// renders.
    // The embedded floor lists every column in `LS_COLUMNS`, so an entry
    // missing here is one a config layer deleted.
    pub columns: BTreeMap<String, ColumnConfig>,
    /// Explicit column order and selection: when set, only the listed
    /// columns render, in that order. Empty (the default) keeps the built-in
    /// column set and order.
    pub columns_order: Vec<String>,
    /// Glyph, colour and label for the ready and blocked states in the `READY`
    /// column. An unset key takes the shipped default.
    pub ready: ReadyConfig,
    /// Colour for the overdue due-date accent. Defaults to `"red"`.
    pub overdue_color: Option<String>,
    /// Ordered auto-activation rules applied to a bare `mt ls` (no CLI
    /// filter, no `@view`): the first rule whose condition holds (or that
    /// has none) wins and supplies its filter, sort, and columns.
    pub rules: Vec<LsRule>,
    /// Whether the `#` and `file` cells carry terminal hyperlinks: `auto`,
    /// `always`, or `never` (the default). `--hyperlinks` overrides it for one
    /// run.
    pub hyperlinks: HyperlinkChoice,
    /// URI that a row's `#` and `file` cells link to. `mt` replaces `{abs}`
    /// with the task's absolute path and `{rel}` with the path the `file`
    /// column shows. Defaults to `file://{abs}`. Point it at another scheme,
    /// such as an editor's URI handler, to open the task there.
    pub hyperlink_target: Option<String>,
    /// Fields that `contains()` searches when a call names none, in order.
    /// Takes any filter field name, plus `body`, `note` and `tags`. Empty (the
    /// default) searches `title`, `id` and `body`. A call that names its
    /// field, such as `contains("x", note)`, ignores this key. An unknown name
    /// is a config error.
    // `deserialize_contains_fields` checks each name against
    // `crate::model::fields::is_contains_field` at config load.
    #[serde(deserialize_with = "deserialize_contains_fields")]
    pub contains_fields: Vec<String>,
}

/// Deserialize `[ls] contains-fields`, rejecting any name outside
/// `crate::model::fields::is_contains_field`'s vocabulary — the same one an explicit
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
        if !crate::model::fields::is_contains_field(name) {
            return Err(serde::de::Error::custom(format!(
                "ls.contains-fields: unknown field `{name}`, expected one of {}",
                crate::model::fields::contains_field_names().join(", ")
            )));
        }
    }
    Ok(names)
}

/// `[ls] hyperlinks` — whether the `#` and `file` cells carry OSC 8
/// hyperlinks to the row's `hyperlink-target`. The `--hyperlinks` flag takes
/// the same values.
///
/// A terminal without OSC 8 support can print the escape sequence as visible
/// text, and no terminal reports whether it supports OSC 8. That is why the
/// default is `never`.
// The CLI crate mirrors this enum one-to-one as
// `crate::render::link::HyperlinkChoice`, because only that crate depends on
// `clap`.
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

impl Facet {
    /// Every facet, in the order the trailer and the docs name them. What a
    /// bare `import-views` entry selects, and what an explicit `@view` filter
    /// argument overrides.
    pub const ALL: [Self; 3] = [Self::Filter, Self::Sort, Self::Columns];
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
            facets: Facet::ALL.to_vec(),
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
/// predicate — see [`crate::filter::condition::Condition::parse`]); each
/// `import-views` entry parses and names a declared view; a manual `filter`
/// parses and its `@view` / `in_view` references resolve; a manual `sort` /
/// `columns` entry is either a legal spec or an `@view` reference that
/// resolves. Returns one message per problem, prefixed `ls.rules[<i>]`.
///
/// A rule can only ever be a *sink* in the view-reference graph (nothing can
/// `import-views` or `@`-reference a rule back), so it cannot participate in
/// a cycle; [`crate::config::views::validate_views`] already guarantees the
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
        && let Err(err) = crate::filter::condition::Condition::parse(condition)
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

/// `[ls.ready]` — glyph, colour and label for the ready and blocked states in
/// the `READY` column. An omitted key takes the shipped default.
// One block covers both states, outside `[ls.columns.ready]`, because
// `ColumnConfig` carries only `when`. An omitted key falls back to the
// embedded floor, then to the renderer's literal fallback.
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
#[path = "ls_tests.rs"]
mod tests;
