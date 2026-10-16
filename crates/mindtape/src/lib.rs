//! Mindtape domain library: a per-file, daemon-free task tracker over Typst.
//!
//! Tasks are one-per-file Typst artifacts (`#show: task.with(...)`). Mindtape
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

pub mod allow;
pub mod at;
pub mod body;
pub mod commit;
pub mod config;
mod date;
pub mod emit;
pub mod filter;
pub mod format;
pub mod framework;
pub mod fs_atomic;
pub mod git;
pub mod graph;
pub mod import_map;
pub mod link;
pub mod loader;
pub mod model;
pub mod plural;
pub mod problem;
pub mod recur;
pub mod rules;
pub mod shell;
pub mod sort;
pub mod suggest;
pub mod template;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod validate;
pub mod waits;
pub mod worktrees;

pub use at::{AtConfig, AtEntry, AtError, AtFs, DiskFs, Forks, Member, Target};
pub use body::content_gate::{CompiledContentRule, check_content, compile_content_rules};
pub use body::gate::{CompiledRule, GateError, check_headers, compile_rules};
pub use body::{BodyRules, check_body_rules};
pub use commit::expr::CommitExpr;
pub use commit::range::{CommitRange, PastCommit, RangeCommit};
pub use commit::tasks::{CommitStates, ProjectState, StatePlan, StateSource};
pub use commit::tip::check_tip;
pub use commit::{
    ChangedFiles, CommitMessage, CommitMsgError, CommitRecord, RefProblem, TrailerValue,
    parse_message, staged_files,
};
pub use config::active_rules::{ActiveRule, RuleSection, active_rules};
pub use config::comments::{CommentsConfig, CommentsError, CommentsPolicy, check_comments};
pub use config::commits::{CommitBreach, CommitRuleSpec, CommitScope, CommitsConfig, check_commit};
pub use config::layers::{
    LayerSource, load_config_layered, load_config_layered_from, merged_config_table,
};
pub use config::packs::{PackOrigin, RuleOrigin, RuleOriginEntry};
pub use config::schema::{SCHEMA_URL, config_schema_json};
pub use config::{
    Actor, ActorKind, AddConfig, AssigneeConfig, BUILTIN_PREFIX, Collector, ColumnConfig, Config,
    ConfigError, DeriveConfig, DeriveTags, Dict, DictCase, DictSpec, Dicts, EstimateConfig, Facet,
    FallbackSeverity, FormatConfig, GoConfig, HyperlinkChoice, ImportView, InteractiveMode,
    LS_COLUMNS, LocalPackage, LsRule, PriorityConfig, ReadyConfig, RetireConfig, RmConfig,
    RmStrategy, Role, RootConfig, RootSpec, TemplatesConfig, TypstConfig, ViewSpec,
    builtin_dict_names, compile_dicts, config_path, embedded_defaults, load_config, parse_config,
    parse_import_view, roles_of, validate_dicts, validate_views,
};
pub use date::pin::{DateOverrideError, init_pinned_today};
pub use date::{DATE_TOKENS, DateToken, days_ago_iso, format_iso, parse_ymd, today_iso, today_ymd};
pub use emit::emit_task;
pub use emit::format::{typ_array, typ_str};
pub use emit::status_value::{STATUS_SHORTCUTS, escape_markup, status_value};
pub use filter::condition::{Condition, ConditionError};
pub use filter::condition_env::{ColumnFacts, ConditionEnv};
pub use filter::{DictMiss, Filter, FilterError};
pub use format::format_typst;
pub use framework::{Category, Framework, Input, InputKind};
pub use fs_atomic::write_atomic;
pub use git::current_branch;
pub use graph::children::{children_violation, unfinished_children};
pub use graph::ready::{
    Blocker, Readiness, ReadyState, blockers_of, unblocked_violation, waits_on,
};
pub use graph::{Cycle, Edge, edges, find_referrer, resolve_edge, toposort};
pub use import_map::{
    EntryKind, ImportReport, MapOptions, Mapped, ReportEntry, map_canonical, slug_from_title,
    stem_for_title,
};
pub use link::scan::{EdgeCall, TargetSpan, link_edges, link_target_spans};
pub use loader::app_dirs::{CONFIG_HOME_ENV, DATA_HOME_ENV, config_home, data_home};
pub use loader::derive::RootDeriver;
pub use loader::{
    LoadError, LoadOutcome, LoadWarning, find_root, load_project, package_overrides, package_root,
};
pub use model::canonical::{CanonicalTask, SCHEMA_VERSION};
pub use model::handle::{Resolution, StemHashes};
pub use model::marker::ShapeError;
pub use model::numeric::{
    Anchor, Band, InputValue, NumCell, NumValue, NumericField, ScaleKind, format_num,
};
pub use model::reference::{Base, RefError, Resolved, Via, is_path_ref, resolve_ref};
pub use model::task_spans::TaskCallSpans;
pub use model::{Status, Task, is_history_pointer, normalize_link, relative_target, slash_path};
pub use plural::{count, plural};
pub use problem::Problem;
pub use rules::{CheckSpec, RuleSpec};
pub use sort::{default_order, default_order_with, difficulty_value, sort_by_specs};
pub use suggest::{Unknown, nearest};
pub use template::add_slots::{RequiredSlot, inherit_field_choices, required_core_slots};
pub use template::check::{
    TemplateHome, TemplateSources, code_span_examples, template_breaches, template_failure,
    unfillable_slots,
};
pub use template::{FillError, RawValue, SlotKind, SlotSpec, discover_slots, fill, fill_examples};
pub use validate::candidate::{
    Candidate, Checked, check_candidate, check_candidates, harvest_candidate, reads_set_breach,
};
pub use validate::severity::Severity;
pub use validate::{Location, Violation, ViolationKind, validate};
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
