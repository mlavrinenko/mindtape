//! `MindTape` domain library: a per-file, daemon-free task tracker over Typst.
//!
//! Tasks are one-per-file Typst artifacts (`#show: task.with(...)`). `MindTape`
//! discovers them under the configured roots, live-evaluates each via
//! [`typst_harvest`], projects the harvested marker into a [`Task`], and offers
//! a filter-expression query language and config-driven validation.
//!
//! The CLI is a thin wrapper over this crate; other tools (e.g. speconaut) can
//! depend on it directly.

// `disallowed-methods` in `clippy.toml` bans `std::fs::write`/`File::create`
// outside `fs_atomic::write_atomic`; unit tests (this crate's own `cfg(test)`
// build) build throwaway fixtures with the plain calls instead of the
// primitive they exist to pin.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

pub mod add_slots;
pub mod app_dirs;
mod body_content;
pub mod body_gate;
mod body_headers;
pub mod candidate;
pub mod canonical;
pub mod children;
pub mod commit_expr;
pub mod commit_msg;
pub mod commit_range;
pub mod commit_refs;
pub mod commit_task_expr;
pub mod commit_tasks;
pub mod condition;
pub mod condition_env;
pub mod config;
mod config_actors;
mod config_add;
mod config_assignee;
pub mod config_comments;
pub mod config_commits;
mod config_dicts;
mod config_estimate;
mod config_format;
mod config_go;
pub mod config_hooks;
mod config_layers;
mod config_links_validate;
mod config_ls;
mod config_ls_columns;
mod config_packs;
mod config_packs_toml;
mod config_required;
mod config_rm;
mod config_roots;
pub mod config_rules;
mod config_rules_typst;
mod config_schema;
mod config_status;
mod config_tasks;
mod config_views;
pub mod content_gate;
mod date;
mod date_pin;
mod dep_rank;
pub mod derive;
pub mod emit;
mod expr;
mod field_descriptor;
mod fields;
pub mod filter;
mod filter_aggregate;
mod filter_compare;
mod filter_links;
mod filter_predicates;
mod filter_strings;
mod filter_views;
pub mod format;
pub mod framework;
mod framework_eval;
mod framework_validate;
pub mod fs_atomic;
pub mod gate;
pub mod git;
pub mod git_state;
pub mod graph;
pub mod handle;
pub mod import_map;
pub mod link;
pub mod link_scan;
pub mod loader;
pub mod marker;
mod merge;
pub mod model;
mod name_cycle;
pub mod naming;
pub mod numeric;
pub mod ready;
pub mod recur;
pub mod recur_dates;
#[cfg(feature = "rrule")]
mod recur_rrule;
pub mod task_spans;
// Without the feature, the same module name resolves to a stub with the same
// surface — so every caller stays one always-compiled function, with no
// `#[cfg]` twin that a mutation sweep can only ever see as dead code.
#[cfg(not(feature = "rrule"))]
#[path = "recur_rrule_off.rs"]
mod recur_rrule;
pub mod severity;
pub mod sort;
pub mod status_value;
pub mod suggest;
pub mod template;
pub mod template_check;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod validate;
mod validate_links;
mod validate_priority;
mod validate_recur;
mod violation;
pub mod worktrees;

pub use add_slots::{RequiredSlot, inherit_field_choices, required_core_slots};
pub use app_dirs::{CONFIG_HOME_ENV, DATA_HOME_ENV, config_home, data_home};
pub use body_gate::{BodyRules, check_body_rules};
pub use candidate::{
    Candidate, Checked, check_candidate, check_candidates, harvest_candidate, reads_set_breach,
};
pub use canonical::{CanonicalTask, SCHEMA_VERSION};
pub use children::{children_violation, unfinished_children};
pub use commit_expr::CommitExpr;
pub use commit_msg::{
    ChangedFiles, CommitMessage, CommitMsgError, CommitRecord, RefProblem, TrailerValue,
    parse_message, staged_files,
};
pub use commit_range::{CommitRange, PastCommit, RangeCommit};
pub use commit_tasks::{CommitStates, ProjectState, StatePlan, StateSource};
pub use condition::{Condition, ConditionError};
pub use condition_env::{ColumnFacts, ConditionEnv};
pub use config::{
    Actor, ActorKind, AddConfig, AssigneeConfig, BUILTIN_PREFIX, Collector, ColumnConfig, Config,
    ConfigError, DeriveConfig, DeriveTags, Dict, DictCase, DictSpec, Dicts, EstimateConfig, Facet,
    FallbackSeverity, FormatConfig, GoConfig, HyperlinkChoice, ImportView, InteractiveMode,
    LS_COLUMNS, LocalPackage, LsRule, PriorityConfig, ReadyConfig, RetireConfig, RmConfig,
    RmStrategy, Role, RootConfig, RootSpec, TemplatesConfig, TypstConfig, ViewSpec,
    builtin_dict_names, compile_dicts, config_path, embedded_defaults, load_config, parse_config,
    parse_import_view, roles_of, validate_dicts, validate_views,
};
pub use config_comments::{CommentsConfig, CommentsError, CommentsPolicy, check_comments};
pub use config_commits::{CommitBreach, CommitRuleSpec, CommitScope, CommitsConfig, check_commit};
pub use config_layers::{
    LayerSource, load_config_layered, load_config_layered_from, merged_config_table,
};
pub use config_packs::{PackOrigin, RuleOrigin, RuleOriginEntry};
pub use config_rules::{CheckSpec, RuleSpec};
pub use config_schema::{SCHEMA_URL, config_schema_json};
pub use content_gate::{CompiledContentRule, check_content, compile_content_rules};
pub use date::{days_ago_iso, format_iso, parse_ymd, today_iso, today_ymd};
pub use date_pin::{DateOverrideError, init_pinned_today};
pub use derive::RootDeriver;
pub use emit::emit_task;
pub use emit::format::{typ_array, typ_str};
pub use filter::{Filter, FilterError};
pub use format::format_typst;
pub use framework::{Category, Framework, Input, InputKind};
pub use fs_atomic::write_atomic;
pub use gate::{CompiledRule, GateError, check_headers, compile_rules};
pub use git::current_branch;
pub use graph::{Cycle, Edge, edges, find_referrer, resolve_edge, toposort};
pub use handle::{RefError, Resolution, Resolved, StemHashes, is_path_ref, resolve_ref};
pub use import_map::{
    EntryKind, ImportReport, MapOptions, Mapped, ReportEntry, map_canonical, slug_from_title,
    stem_for_title,
};
pub use link_scan::{EdgeCall, TargetSpan, link_edges, link_target_spans};
pub use loader::{
    LoadError, LoadOutcome, find_root, load_project, package_overrides, package_root,
};
pub use marker::ShapeError;
pub use model::{Status, Task, is_history_pointer, normalize_link};
pub use numeric::{
    Anchor, Band, InputValue, NumCell, NumValue, NumericField, ScaleKind, format_num,
};
pub use ready::{Blocker, Readiness, ReadyState, blockers_of, unblocked_violation, waits_on};
pub use severity::Severity;
pub use sort::{default_order, default_order_with, difficulty_value, sort_by_specs};
pub use status_value::{STATUS_SHORTCUTS, escape_markup, status_value};
pub use suggest::{hint, nearest};
pub use task_spans::TaskCallSpans;
pub use template::{FillError, RawValue, SlotKind, SlotSpec, discover_slots, fill, fill_examples};
pub use template_check::{
    TemplateHome, TemplateSources, template_breaches, template_failure, unfillable_slots,
};
pub use validate::{Location, Violation, validate};
pub use worktrees::{WorktreeEntry, WorktreeError, WorktreeRegistry};

/// Retain the tasks matching `filter`; `None` keeps them all. `config` resolves
/// framework-scored priorities for numeric comparisons; `readiness` answers the
/// `ready` / `blocked` predicates and must be computed over the full set before
/// any narrowing (pass [`Readiness::empty`] when no set context applies).
///
/// # Errors
/// Returns [`FilterError`] if the filter references an unknown field.
pub fn select(
    tasks: Vec<Task>,
    filter: Option<&Filter>,
    config: &Config,
    readiness: &Readiness,
) -> Result<Vec<Task>, FilterError> {
    let Some(filter) = filter else {
        return Ok(tasks);
    };
    let mut kept = Vec::new();
    for task in tasks {
        if filter.matches_with(&task, config, readiness)? {
            kept.push(task);
        }
    }
    Ok(kept)
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
