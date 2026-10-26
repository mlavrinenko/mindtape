//! `.mindtape/config.toml` loading and the configurable task schema.
//!
//! The config is a white-list: Mindtape constrains only what is declared here
//! (directly or through the embedded floor, `defaults.toml`). A handful of
//! sections carry an opinionated shipped default — the status legend, the
//! priority/difficulty scale and classification, `roots`, `[templates]`,
//! `[rm]`/`[ls]` default filters — and omitting one of those inherits the
//! floor rather than going unconstrained. Set it explicitly empty (`status =
//! []`, `[tasks.priority] bands = []`) to unconstrain it instead. Every other
//! section (estimate units, tag allow-list, body headers, derive-tags) stays
//! unconstrained when omitted, as before.
//!
//! # Files
//!
//! - `active_rules.rs`: every rule the merged config applies, for `mt rules`.
//! - `actors.rs`: `[[actors]]` and `[[roles]]`, the actor registry.
//! - `add.rs`: `[add]`, the interactive editor fill for `mt add`.
//! - `assignee.rs`: `[tasks.assignee]` constraints.
//! - `comments.rs`: `[tasks.comments]`, policy over review comments.
//! - `commits.rs`: `[commits]` and `[[commits.rules]]`.
//! - `defaults.toml`: the embedded floor every config layers over.
//! - `dicts.rs`: `[dicts.<name>]`, named string sets.
//! - `estimate.rs`: `[tasks.estimate]` unit allow-list.
//! - `expression_keys.rs`: every expression key, read from the config schema.
//! - `format.rs`: `[format]`, whether `mt` reformats what it writes.
//! - `go.rs`: `[go]`, the editor `mt go` opens.
//! - `hooks.rs`: `[[on.flip]]`, actions bound to a status change.
//! - `layers.rs`: multi-file config layer discovery and merge.
//! - `links_validate.rs`: per-kind link-target validators.
//! - `ls.rs`: `[ls]` rendering defaults.
//! - `ls_columns.rs`: `[ls.columns.<name>]` and the column-name vocabulary.
//! - `locate.rs`: the file line of a dotted config key.
//! - `merge.rs`: the shared TOML deep merge.
//! - `name_cycle.rs`: cycle detection over a name-reference graph.
//! - `pack_dirs.rs`: a directory pack's `templates/`, appended to `templates.dirs`.
//! - `pack_files.rs`: a pack's check files and dictionary paths, resolved from its directory.
//! - `pack_fold.rs`: one layer's packs, folded into the sub-layer below it.
//! - `pack_registry.rs`: the packs compiled into the binary and the origin types.
//! - `pack_resolve.rs`: resolving one `extends` entry to a validated pack table.
//! - `pack_sources/`: `[pack-sources]` and `.mindtape/packs.lock`.
//! - `packs.rs`: `extends` and `disable`, merging packs into each layer.
//! - `packs_toml.rs`: nested-path reads and writes over a raw `toml::Table`.
//! - `params.rs`: `[params]` and `param()`, a pack's parameters and their rewrite.
//! - `priority.rs`: `[tasks.priority]`, the priority scale and its frameworks.
//! - `required.rs`: `required`, whether a core field must be set.
//! - `rm.rs`: `[rm]` and `[retire]`, removal and retirement policy.
//! - `roots.rs`: task root specs and the folder-derived-tag policy.
//! - `schema.rs`: the JSON Schema derived from the config structs.
//! - `semantic.rs`: `Config::validate`, the checks that go beyond TOML shape.
//! - `status.rs`: `[[tasks.status]]`, the status legend.
//! - `tasks.rs`: the `[tasks]` section struct and its small field types.
//! - `templates.rs`: `[templates]`, where `mt add` finds task templates.
//! - `typst.rs`: `[typst]` and its project-local packages.
//! - `views.rs`: `[views]`, named query bundles.
//! - `warnings.rs`: load-time warnings about a config that loads but looks wrong.

pub mod active_rules;
pub(crate) mod actors;
pub(crate) mod add;
pub(crate) mod assignee;
pub mod comments;
pub mod commits;
pub(crate) mod dicts;
pub(crate) mod estimate;
pub(crate) mod expression_keys;
pub(crate) mod format;
pub(crate) mod go;
pub mod hooks;
pub(crate) mod layers;
pub(crate) mod links_validate;
mod locate;
pub(crate) mod ls;
pub(crate) mod ls_columns;
pub(crate) mod merge;
pub(crate) mod name_cycle;
pub(crate) mod pack_dirs;
pub(crate) mod pack_files;
pub(crate) mod pack_fold;
pub(crate) mod pack_registry;
pub(crate) mod pack_resolve;
pub mod pack_sources;
pub(crate) mod packs;
pub(crate) mod packs_toml;
pub(crate) mod params;
pub(crate) mod priority;
pub(crate) mod required;
pub(crate) mod rm;
pub(crate) mod roots;
pub(crate) mod schema;
mod semantic;
pub(crate) mod status;
pub(crate) mod tasks;
pub(crate) mod templates;
pub(crate) mod typst;
pub(crate) mod views;
pub(crate) mod warnings;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::config::merge::deep_merge_tables;
use crate::framework::merge_builtin_frameworks;
use crate::problem::Problem;
use crate::suggest::Unknown;

pub use crate::body::content::{
    ContentAssert, ContentConfig, ContentRuleConfig, DensitySpec, DensityUnit,
};
pub use crate::body::headers::{HeaderAction, HeaderPathSegment, HeaderRuleConfig, HeadersConfig};
pub use crate::config::add::{AddConfig, InteractiveMode};
pub use crate::config::dicts::{
    BUILTIN_PREFIX, Dict, DictCache, DictCase, DictSpec, Dicts, builtin_dict_names, compile_dicts,
    validate_dicts,
};
pub use crate::config::format::FormatConfig;
pub use crate::config::go::GoConfig;
pub use crate::config::locate::ConfigPlace;
pub(crate) use crate::config::locate::quote;
pub use crate::config::ls::{
    Facet, HyperlinkChoice, ImportView, LS_COLUMNS, LsConfig, LsRule, ReadyConfig,
    parse_import_view, validate_ls_rules,
};
pub use crate::config::ls_columns::{ColumnConfig, validate_ls_columns};
pub use crate::config::params::{PackEntry, PackWith, ParamValue};
pub use crate::config::priority::PriorityConfig;
pub use crate::config::required::{RequiredRules, Requirement, RequirementRule};
pub use crate::config::rm::{RetireConfig, RmConfig, RmStrategy};
pub use crate::config::roots::{Collector, DeriveConfig, DeriveTags, RootConfig, RootSpec};
pub use crate::config::status::{Legend, LegendEntry};
pub use crate::config::tasks::{
    FieldConfig, IdConfig, IdFormat, TagsConfig, TaskBodyConfig, TasksConfig,
};
pub use crate::config::templates::{FallbackSeverity, TemplatesConfig};
pub use crate::config::typst::{LocalPackage, TypstConfig};
pub use crate::config::views::{ViewSpec, validate_view_statuses, validate_views};

/// The embedded config floor: canonical default values for the sections that
/// carry an opinionated shipped default (status legend, priority/difficulty
/// scale and classification, roots, templates, `rm`/`ls` defaults). Deep-merged
/// as the base under the user's config before typed deserialization — see
/// [`parse_config`]. Mirrors `BUILTIN_FRAMEWORKS` (`framework/mod.rs`). Also the
/// lowest layer in [`crate::config::layers`]'s multi-file assembly.
pub(crate) const EMBEDDED_DEFAULTS: &str = include_str!("defaults.toml");

/// The embedded default floor's raw TOML text, verbatim. Backs
/// `mt config show --defaults`, so a user seeding an XDG or project override
/// sees the shipped baseline exactly as compiled in, not a re-serialised copy.
#[must_use]
pub fn embedded_defaults() -> &'static str {
    EMBEDDED_DEFAULTS
}

/// Top-level project configuration.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Project-level settings (code, etc.).
    pub project: ProjectConfig,
    /// Task discovery, status lifecycle, and field constraints.
    pub tasks: TasksConfig,
    /// `mt ls` defaults.
    pub ls: LsConfig,
    /// `rm` (task removal) defaults.
    pub rm: RmConfig,
    /// Settings for the gitless `rm` strategy's tombstone.
    pub retire: RetireConfig,
    /// `add` (task creation) template discovery.
    pub templates: TemplatesConfig,
    /// `add` interactive-fill settings (editor, when it opens).
    pub add: AddConfig,
    /// `go` (open in editor) settings.
    pub go: GoConfig,
    /// Typst evaluation settings (local-package overrides).
    pub typst: TypstConfig,
    /// Whether `mt` reformats the Typst source it writes (`[format]`).
    pub format: FormatConfig,
    /// Named query bundles (`[views.<name>]`), referenced by `@name`.
    pub views: BTreeMap<String, ViewSpec>,
    /// Named string sets (`[dicts.<name>]`), so a vocabulary two rules share is
    /// one list rather than two that drift. Each unions `values`, `path` and
    /// `include` into one set, compiled and checked once at config load. The
    /// `mindtape-` prefix is reserved for the built-in dictionaries.
    pub dicts: BTreeMap<String, DictSpec>,
    /// [`Config::dicts`] compiled — the resolved entry sets the filter
    /// language's `in_dict()` probes. Derived, never declared: it carries no
    /// TOML key and no schema property, and it is the one field
    /// [`PartialEq`] ignores (see [`DictCache`]). Read it through
    /// [`Config::compiled_dicts`].
    #[serde(skip)]
    pub(crate) dict_cache: DictCache,
    /// The project root this config was built for, resolved once by
    /// [`crate::config::layers::build_config`]. Derived, never declared: like
    /// [`Config::dict_cache`], it carries no TOML key, no schema property, and
    /// is never equal-compared. Read by a `[[tasks.rules]] check` entry
    /// ([`crate::rules::typst`]) instead of taking `root` as a parameter
    /// on every validation call site.
    #[serde(skip)]
    pub(crate) rules_root: crate::rules::RulesRoot,
    /// Where each `[[tasks.rules]]` entry is declared, recorded by
    /// [`crate::config::layers::build_config`]. Derived, never declared, never
    /// equal-compared — see [`crate::rules::sites::RuleSites`].
    #[serde(skip)]
    pub(crate) rule_sites: crate::rules::sites::RuleSites,
    /// The editor commands the user's own config layer sets, which `mt allow`
    /// does not ask about. Derived at load, never declared, never
    /// equal-compared — see [`crate::allow::UserCommands`].
    #[serde(skip)]
    pub(crate) user_commands: crate::allow::UserCommands,
    /// What git says changed under the project root, read the first time a
    /// filter's `changed()` asks and shared from then on. Derived, never
    /// declared, never equal-compared — see [`crate::git::state::GitCache`].
    #[serde(skip)]
    pub(crate) git: crate::git::state::GitCache,
    /// The task trailer, and the rules `mt check --commit-msg` checks a commit
    /// against.
    pub commits: crate::config::commits::CommitsConfig,
    /// Status-transition hooks (`[[on.flip]]`), run only by `mt flip`.
    pub on: crate::config::hooks::OnConfig,
    /// The project's actor registry (`[[actors]]`): named workers (people and
    /// language models) that `assignee:` values may reference.
    pub actors: Vec<crate::config::actors::Actor>,
    /// Role names (`[[roles]]`) that actors may hold. A role carries no
    /// settings; a harness keys its own config by role name.
    pub roles: Vec<crate::config::actors::Role>,
    /// Packs this config builds on, in any config layer. An entry with a path
    /// separator is a TOML pack file, or a directory pack, a folder holding
    /// `pack.toml` and an optional `templates/`, relative to the file that
    /// declares it. A directory's `templates/` joins `templates.dirs`.
    /// Any other entry is `<source>:<pack>`, and `mindtape` is the one source
    /// known without a declaration: `mindtape:tasks` is a pack built into
    /// `mt`. A pack holds any config key but `extends`, `disable`,
    /// `pack-sources`, `project.code`, `tasks.roots`, `templates.dirs` and
    /// `[typst]`. Its
    /// keys merge just below this layer's own, table by table, and its rules
    /// come ahead of this layer's, where a rule with the same name replaces
    /// it. Two packs here that set one value differently are an error,
    /// unless this layer sets that key too. An entry `{ pack, with }` sets
    /// the pack's parameters, which its expressions read as
    /// `param("name")`.
    // `crate::config::packs` resolves and consumes `extends` before this struct
    // is built, so the field reads back as whichever layer declared it last.
    // No command reads it.
    pub extends: Vec<PackEntry>,
    /// Rule names to drop once every pack and layer has added its rules. A
    /// name that matches no rule is a config error.
    pub disable: Vec<String>,
    /// Git repositories the project takes packs from, each a
    /// `[pack-sources.<name>]` table of `git`, the repository, and `ref`, a
    /// tag, branch or commit. An `extends` entry `<name>:<pack>` names a pack
    /// in one. Only the project's own `.mindtape/config.toml` declares a
    /// source, and `mindtape` is taken. `.mindtape/packs.lock` pins each
    /// source to a commit, and a config load refuses a source the lock
    /// doesn't pin from the `git` and `ref` declared here.
    #[serde(rename = "pack-sources")]
    pub pack_sources: BTreeMap<String, crate::config::pack_sources::PackSource>,
}
/// Project-level settings.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ProjectConfig {
    /// Short project qualifier. Falls back to the root directory name when absent.
    pub code: Option<String>,
}

/// Resolve the effective project code: configured `code` or the root dir name.
#[must_use]
pub fn effective_project_code(config: &Config, root: &std::path::Path) -> String {
    if let Some(code) = &config.project.code {
        return code.clone();
    }
    root.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub use crate::config::actors::{Actor, ActorKind, Role, roles_of};
pub use crate::config::assignee::AssigneeConfig;
pub use crate::config::estimate::EstimateConfig;

/// Errors from loading or locating a config file.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The config file could not be read.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// The config file contained invalid TOML. The TOML error is the message,
    /// not a source, so a caller printing the chain prints it once.
    #[error("parse error: {0}")]
    Parse(toml::de::Error),
    /// A config layer or pack other than the project's own contained invalid
    /// TOML. `origin` names the file, or `pack <name>` for an embedded pack.
    #[error("parse error in {origin}: {error}")]
    ParseIn {
        /// The file path, or `pack <name>`, the TOML error is in.
        origin: String,
        /// The TOML error, in the message rather than a `source` link, so a chain prints it once.
        error: toml::de::Error,
    },
    /// The config parsed but failed a semantic check (e.g. band coverage).
    #[error("invalid config: {message}")]
    Invalid {
        /// The problems, one per line, each led by the dotted key it is about.
        message: String,
        /// Where the first problem's key sits, when a config file declares it.
        place: Option<ConfigPlace>,
        /// What the first problem's unknown name could have been, if it has
        /// one.
        unknown: Option<Unknown>,
        /// What to try next, when the first problem has an obvious next step.
        hint: Option<&'static str>,
    },
}

impl ConfigError {
    /// A semantic failure not yet tied to a file.
    #[must_use]
    pub fn invalid(problem: impl Into<Problem>) -> Self {
        let problem = problem.into();
        Self::Invalid {
            message: problem.text,
            place: None,
            unknown: problem.unknown,
            hint: None,
        }
    }

    /// `self` with `next` as what to try next, when it is
    /// [`ConfigError::Invalid`].
    #[must_use]
    pub fn with_hint(mut self, next: &'static str) -> Self {
        if let Self::Invalid { hint, .. } = &mut self {
            *hint = Some(next);
        }
        self
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(error: toml::de::Error) -> Self {
        Self::Parse(error)
    }
}

impl Config {
    /// What loads fine but is likely a mistake: a stored filter naming an `x-`
    /// field that `[tasks.fields]` does not declare, beside declared ones. One
    /// message per finding, prefixed by the dotted TOML path of the filter.
    #[must_use]
    pub fn warnings(&self) -> Vec<crate::problem::Problem> {
        crate::config::warnings::warnings(self)
    }

    /// The compiled `[dicts.<name>]` sets, plus the built-ins, which are always
    /// present — so `in_dict("mindtape-verbs", ...)` resolves against a project
    /// declaring no `[dicts]` section at all. A name absent here is an unknown
    /// dictionary, reported by the caller that wrote it
    /// ([`crate::filter::FilterError::UnknownDict`]), never a silent miss.
    #[must_use]
    pub fn compiled_dicts(&self) -> &Dicts {
        self.dict_cache.dicts()
    }

    /// The stems this command's `touches()` calls met that no task answers
    /// to, even through a rename — see [`crate::commit::refs`].
    #[must_use]
    pub fn retired_refs(&self) -> Vec<String> {
        self.git.retired_refs()
    }

    /// The view that drives the default `mt ls` listing when the caller passes
    /// no filter: `[ls] default-view` by name if set, else the implicit
    /// `[views.default]` view, else `None` (match-all). `Some` here supplies the
    /// default listing's filter, sort, and columns as a bundle. Validated at
    /// load, so a set `default-view` name always resolves.
    #[must_use]
    pub fn ls_default_view(&self) -> Option<&ViewSpec> {
        let name = self.ls.default_view.as_deref().unwrap_or("default");
        self.views.get(name)
    }
}

/// Parse a config from a TOML string.
///
/// The embedded floor ([`EMBEDDED_DEFAULTS`]) is deep-merged as the base, the
/// user's text as the overlay (tables merge field-by-field; a user section
/// wins wholesale over the floor's for arrays and scalars — an explicit
/// `status = []` unconstrains a floored section rather than being ignored).
/// Built-in priority frameworks are then deep-merged into
/// `[tasks.priority.frameworks]`, so a user config can activate (and
/// selectively override) a built-in without redeclaring it. Order matters:
/// floor, then user, then frameworks.
///
/// `[dicts]` is compiled here too, rootless
/// ([`crate::config::dicts::compile_dicts_rootless`]), so a config parsed from a
/// string alone still answers `in_dict()`. This function is shape-only —
/// semantic checks are [`Config::validate`]'s — so a section that fails to
/// compile leaves the built-in dictionaries alone rather than erroring; the
/// real message comes from `validate` or from config load, which re-compiles
/// against the project root.
///
/// # Errors
/// Returns [`ConfigError::Parse`] if the TOML is invalid (the user's text, or,
/// were it ever malformed, the embedded floor itself).
pub fn parse_config(text: &str) -> Result<Config, ConfigError> {
    let mut doc: toml::Table = toml::from_str(EMBEDDED_DEFAULTS)?;
    let overlay: toml::Table = toml::from_str(text)?;
    deep_merge_tables(&mut doc, overlay);
    merge_builtin_frameworks(&mut doc);
    let mut config: Config = toml::Value::Table(doc).try_into()?;
    config.dict_cache = crate::config::dicts::compile_dicts_rootless(&config.dicts)
        .map(DictCache::from)
        .unwrap_or_default();
    Ok(config)
}

/// Load the effective config for `root`: the embedded floor, the XDG global
/// defaults, every ancestor `.mindtape/defaults.toml`, and the project's own
/// `.mindtape/config.toml`, deep-merged in that precedence order. The `.0` of
/// [`crate::config::layers::load_config_layered`], which also returns each
/// layer's provenance (for `mt config show --origin`); see that module for the
/// full assembly.
///
/// # Errors
/// Returns [`ConfigError`] if a layer exists but cannot be read or parsed, or
/// the merged config fails semantic validation.
pub fn load_config(root: &Path) -> Result<Config, ConfigError> {
    crate::config::layers::load_config_layered(root).map(|(config, _sources, _rule_origins)| config)
}

/// The conventional config path for a project root.
#[must_use]
pub fn config_path(root: &Path) -> PathBuf {
    root.join(".mindtape").join("config.toml")
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_id;

#[cfg(test)]
mod tests_headers;

#[cfg(test)]
mod tests_ls;
