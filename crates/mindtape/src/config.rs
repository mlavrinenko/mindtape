//! `.mindtape/config.toml` loading and the configurable task schema.
//!
//! The config is a white-list: `MindTape` constrains only what is declared here
//! (directly or through the embedded floor, `defaults.toml`). A handful of
//! sections carry an opinionated shipped default — the status legend, the
//! priority/difficulty scale and classification, `roots`, `[templates]`,
//! `[rm]`/`[ls]` default filters — and omitting one of those inherits the
//! floor rather than going unconstrained. Set it explicitly empty (`status =
//! []`, `[tasks.priority] bands = []`) to unconstrain it instead. Every other
//! section (estimate units, tag allow-list, body headers, derive-tags) stays
//! unconstrained when omitted, as before.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::framework::merge_builtin_frameworks;
use crate::merge::deep_merge_tables;

pub use crate::body_content::{
    ContentAssert, ContentConfig, ContentRuleConfig, DensitySpec, DensityUnit,
};
pub use crate::body_headers::{HeaderAction, HeaderPathSegment, HeaderRuleConfig, HeadersConfig};
pub use crate::config_add::{AddConfig, InteractiveMode};
pub use crate::config_dicts::{
    BUILTIN_PREFIX, Dict, DictCache, DictCase, DictSpec, Dicts, builtin_dict_names, compile_dicts,
    validate_dicts,
};
pub use crate::config_format::FormatConfig;
pub use crate::config_go::GoConfig;
pub use crate::config_ls::{
    Facet, HyperlinkChoice, ImportView, LS_COLUMNS, LsConfig, LsRule, ReadyConfig,
    parse_import_view, validate_ls_rules,
};
pub use crate::config_ls_columns::{ColumnConfig, validate_ls_columns};
pub use crate::config_required::{RequiredRules, Requirement, RequirementRule};
pub use crate::config_rm::{RetireConfig, RmConfig, RmStrategy};
pub use crate::config_roots::{Collector, DeriveConfig, DeriveTags, RootConfig, RootSpec};
pub use crate::config_status::{Legend, LegendEntry};
pub use crate::config_tasks::{
    FieldConfig, IdConfig, IdFormat, PriorityConfig, TagsConfig, TaskBodyConfig, TasksConfig,
};
pub use crate::config_views::{ViewSpec, validate_views};

/// The embedded config floor: canonical default values for the sections that
/// carry an opinionated shipped default (status legend, priority/difficulty
/// scale and classification, roots, templates, `rm`/`ls` defaults). Deep-merged
/// as the base under the user's config before typed deserialization — see
/// [`parse_config`]. Mirrors `BUILTIN_FRAMEWORKS` (`framework.rs`). Also the
/// lowest layer in [`crate::config_layers`]'s multi-file assembly.
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
    /// `[retire]` — the gitless `rm` strategy's tombstone settings.
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
    /// [`crate::config_layers::build_config`]. Derived, never declared: like
    /// [`Config::dict_cache`], it carries no TOML key, no schema property, and
    /// is never equal-compared. Read by a `[[tasks.rules]] check` entry
    /// ([`crate::config_rules_typst`]) instead of taking `root` as a parameter
    /// on every validation call site.
    #[serde(skip)]
    pub(crate) rules_root: crate::config_rules::RulesRoot,
    /// Where each `[[tasks.rules]]` entry is declared, recorded by
    /// [`crate::config_layers::build_config`]. Derived, never declared, never
    /// equal-compared — see [`crate::config_rule_sites::RuleSites`].
    #[serde(skip)]
    pub(crate) rule_sites: crate::config_rule_sites::RuleSites,
    /// What git says changed under the project root, read the first time a
    /// filter's `changed()` asks and shared from then on. Derived, never
    /// declared, never equal-compared — see [`crate::git_state::GitCache`].
    #[serde(skip)]
    pub(crate) git: crate::git_state::GitCache,
    /// `[commits]` — the task trailer and `[[commits.rules]]`, checked by
    /// `mt check --commit-msg`.
    pub commits: crate::config_commits::CommitsConfig,
    /// Status-transition hooks (`[[on.flip]]`), run only by `mt flip`.
    pub on: crate::config_hooks::OnConfig,
    /// The project's actor registry (`[[actors]]`): named workers (people and
    /// language models) that `assignee:` values may reference.
    pub actors: Vec<crate::config_actors::Actor>,
    /// Role names (`[[roles]]`) that actors may hold. A role carries no
    /// settings; a harness keys its own config by role name.
    pub roles: Vec<crate::config_actors::Role>,
    /// Rule packs this config builds on, in any config layer. A name with the
    /// reserved `mindtape-` prefix and no path separator is a pack built into
    /// `mt`. Anything with a path separator is a TOML pack file, relative to
    /// the file that declares it. A pack adds entries to `[[tasks.rules]]`,
    /// `[[tasks.body.headers.rules]]`, `[[tasks.body.content.rules]]`,
    /// `[[commits.rules]]` and `[dicts]`, ahead of this layer's own. A rule in
    /// this layer replaces a pack rule with the same name.
    // `crate::config_packs` resolves and consumes `extends` before this struct
    // is built, so the field reads back as whichever layer declared it last.
    // No command reads it.
    pub extends: Vec<String>,
    /// Rule names to drop once every pack and layer has added its rules. A
    /// name that matches no rule is a config error.
    pub disable: Vec<String>,
}

/// `[typst]` — Typst evaluation settings.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TypstConfig {
    /// `@local/<name>` packages to load from a directory in the project, so a
    /// task can import a package that is not installed globally. `mt`
    /// registers `@local/mindtape` itself when `<root>/lib` holds the
    /// `mindtape` package; list any other package here.
    pub local_packages: Vec<LocalPackage>,
}

/// `[[typst.local-packages]]` — one `@local/<name>` package loaded from a project directory.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct LocalPackage {
    /// The `@local/<name>` package name.
    pub name: String,
    /// Directory holding the package's `typst.toml`, relative to the project root.
    pub path: String,
}

/// `[templates]` — where `mt add` finds task templates.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct TemplatesConfig {
    /// Directories, relative to the project root, holding `*.typ` task templates.
    pub dirs: Vec<String>,
    /// Template stem `mt add` uses when none is named.
    pub default: String,
    /// How `mt check` reports a breach of the built-in template, which `mt add`
    /// falls back to when no template dir holds `task.typ`. A project's own
    /// templates always report errors.
    pub fallback_severity: FallbackSeverity,
}

/// `[templates] fallback-severity` — how a breach of the built-in template
/// reports.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FallbackSeverity {
    /// Print the breach; `mt check` still succeeds. The default.
    #[default]
    Warning,
    /// Print the breach and fail `mt check`.
    Error,
}

// Left opinionated rather than collapsed to empty, for the same reason as
// `TasksConfig::default`'s `roots`: widely relied on by direct `Config::default()`
// test fixtures (e.g. template resolution/listing tests), and its value already
// agrees with the embedded floor (`defaults.toml`).
impl Default for TemplatesConfig {
    fn default() -> Self {
        Self {
            dirs: vec![".mindtape/templates".to_owned()],
            default: "task".to_owned(),
            fallback_severity: FallbackSeverity::Warning,
        }
    }
}

/// `[project]` — project-level settings.
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

pub use crate::config_actors::{Actor, ActorKind, Role, roles_of};
pub use crate::config_assignee::AssigneeConfig;
pub use crate::config_estimate::EstimateConfig;

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
    /// The config parsed but failed a semantic check (e.g. band coverage).
    #[error("invalid config: {0}")]
    Invalid(String),
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
    pub fn warnings(&self) -> Vec<String> {
        crate::config_warnings::warnings(self)
    }

    /// Run semantic checks that go beyond TOML shape (numeric scale and
    /// classification coverage). Returns one joined message per problem.
    ///
    /// # Errors
    /// Returns the collected problems when the numeric fields are inconsistent.
    pub fn validate(&self) -> Result<(), String> {
        let mut problems = self.tasks.priority.validate("priority");
        problems.extend(self.tasks.difficulty.validate("difficulty"));
        problems.extend(self.tasks.status.transition_problems());
        problems.extend(crate::derive::validate_root_globs(self));
        for spec in &self.ls.default_sort {
            if let Err(msg) = crate::sort::validate_sort_spec(spec) {
                problems.push(format!("ls.default-sort: {msg}"));
            }
        }
        problems.extend(validate_ls_columns(&self.ls));
        problems.extend(validate_views(&self.views));
        problems.extend(validate_dicts(&self.dicts));
        problems.extend(crate::config_tasks::validate_field_names(&self.tasks));
        problems.extend(crate::config_required::validate_requirements(self));
        problems.extend(crate::config_required::validate_values_when(self));
        problems.extend(crate::config_links_validate::validate_link_kinds(self));
        problems.extend(crate::config_rules::validate_rules(self));
        problems.extend(crate::config_rules::validate_rule_names(self));
        problems.extend(crate::config_commits::validate_commits(self));
        problems.extend(crate::body_headers::validate_header_rules(self));
        problems.extend(crate::body_content::validate_content_rules(self));
        problems.extend(crate::config_comments::validate_comments(self));
        problems.extend(crate::config_actors::validate_actors(self));
        problems.extend(crate::config_hooks::validate_flip_hooks(self));
        problems.extend(validate_ls_rules(&self.ls.rules, &self.views));
        if let Some(status) = &self.retire.status
            && !self.tasks.status.contains(status)
        {
            problems.push(format!("retire.status: unknown status `{status}`"));
        }
        if let Some(name) = &self.ls.default_view
            && !self.views.contains_key(name)
        {
            problems.push(format!(
                "ls.default-view: unknown view `{name}`{}",
                crate::suggest::hint("view", name, self.views.keys())
            ));
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems.join("; "))
        }
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
    /// to, even through a rename — see [`crate::commit_refs`].
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
/// ([`crate::config_dicts::compile_dicts_rootless`]), so a config parsed from a
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
    config.dict_cache = crate::config_dicts::compile_dicts_rootless(&config.dicts)
        .map(DictCache::from)
        .unwrap_or_default();
    Ok(config)
}

/// Load the effective config for `root`: the embedded floor, the XDG global
/// defaults, every ancestor `.mindtape/defaults.toml`, and the project's own
/// `.mindtape/config.toml`, deep-merged in that precedence order. The `.0` of
/// [`crate::config_layers::load_config_layered`], which also returns each
/// layer's provenance (for `mt config show --origin`); see that module for the
/// full assembly.
///
/// # Errors
/// Returns [`ConfigError`] if a layer exists but cannot be read or parsed, or
/// the merged config fails semantic validation.
pub fn load_config(root: &Path) -> Result<Config, ConfigError> {
    crate::config_layers::load_config_layered(root).map(|(config, _sources, _rule_origins)| config)
}

/// The conventional config path for a project root.
#[must_use]
pub fn config_path(root: &Path) -> PathBuf {
    root.join(".mindtape").join("config.toml")
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "config_tests_id.rs"]
mod tests_id;

#[cfg(test)]
#[path = "config_tests_headers.rs"]
mod tests_headers;

#[cfg(test)]
#[path = "config_tests_ls.rs"]
mod tests_ls;
