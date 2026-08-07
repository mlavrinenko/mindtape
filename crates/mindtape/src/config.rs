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

use serde::{Deserialize, Deserializer};
use thiserror::Error;

use crate::framework::{Framework, merge_builtin_frameworks};
use crate::merge::deep_merge_tables;
use crate::numeric::NumericField;

pub use crate::config_add::{AddConfig, InteractiveMode};
pub use crate::config_format::FormatConfig;
pub use crate::config_go::GoConfig;
pub use crate::config_ls::{
    ColumnConfig, Facet, HyperlinkChoice, ImportView, LS_COLUMNS, LsConfig, LsRule, ReadyConfig,
    Visibility, parse_import_view, validate_ls_columns, validate_ls_rules,
};
pub use crate::config_required::{RequiredRules, Requirement, RequirementRule};
pub use crate::config_rm::{RetireConfig, RmConfig, RmStrategy};
pub use crate::config_roots::{Collector, DeriveConfig, DeriveTags, RootConfig, RootSpec};
pub use crate::config_status::{Legend, LegendEntry};
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
    /// Status-transition hooks (`[[on.flip]]`), run only by `mt flip`.
    pub on: crate::config_hooks::OnConfig,
    /// The project's actor registry (`[[actors]]`): named workers (people and
    /// language models) that `assignee:` values may reference.
    pub actors: Vec<crate::config_actors::Actor>,
    /// Declared role names (`[[roles]]`) actors may hold. Identity-only: a
    /// role carries no payload; the reader's harness keys its own config by
    /// role name by convention.
    pub roles: Vec<crate::config_actors::Role>,
}

/// `[typst]` — Typst evaluation settings.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TypstConfig {
    /// Extra `@local/<name>` package overrides, each resolved from an in-tree
    /// path so a worktree resolves a foreign package without a global install.
    /// `MindTape`'s own `@local/mindtape` is auto-registered when `<root>/lib` is
    /// the `mindtape` package; this seam covers any other package (e.g. a
    /// consumer's own lib referenced by an `x-<ns>` extras helper).
    pub local_packages: Vec<LocalPackage>,
}

/// `[[typst.local-packages]]` — one `@local/<name>` → in-tree path override.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct LocalPackage {
    /// The `@local/<name>` package name.
    pub name: String,
    /// Lib directory holding the package's `typst.toml`, relative to the root.
    pub path: String,
}

/// `[templates]` — where `mt add` finds task templates.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct TemplatesConfig {
    /// Dirs (relative to the project root) holding `*.typ` task templates.
    pub dirs: Vec<String>,
    /// Template stem `mt add` uses when none is named.
    pub default: String,
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

/// `[tasks]` — where tasks live and how they are constrained.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TasksConfig {
    /// Directories (relative to the project root) holding one task per file.
    pub roots: Vec<RootSpec>,
    /// Project-wide folder-derived-tag policy.
    pub derive_tags: DeriveConfig,
    /// Status legend: legal values in lifecycle order, plus per-value display
    /// (glyph, colour, label) and the `terminal` flag.
    pub status: Legend,
    /// Priority as a numeric-field instance (scale + band classification),
    /// optionally driven by a configured scoring framework.
    pub priority: PriorityConfig,
    /// Difficulty as a numeric-field instance (scale + anchor classification).
    pub difficulty: NumericField,
    /// Constraints for the remaining configurable enum core fields, keyed by
    /// Typst slot name (`kind`).
    pub fields: BTreeMap<String, FieldConfig>,
    /// Assignee-field constraints (allowed actors, required).
    pub assignee: crate::config_assignee::AssigneeConfig,
    /// Estimate units allow-list.
    pub estimate: EstimateConfig,
    /// Tag allow-list.
    pub tags: TagsConfig,
    /// Body-shape constraints (heading gating).
    pub body: TaskBodyConfig,
    /// Task id format.
    pub id: IdConfig,
    /// Task stem naming template (`mt add`/`mt recur` minting) and `mt add`'s
    /// default collision policy (`dedup`).
    pub naming: crate::naming::NamingConfig,
    /// Typed relation edge (`links:`) kind registry.
    pub links: crate::link::LinksConfig,
    /// Recurring-tasks feature gate and policy floor (`mt recur`).
    pub recur: crate::recur::RecurConfig,
    /// Work-in-progress caps: how many tasks may match a filter at once
    /// (`[[tasks.limits]]`, enforced by `mt check`).
    pub limits: Vec<crate::config_limits::LimitSpec>,
    /// Review-comment policy over a task's whole source file (`mt check`).
    pub comments: crate::config_comments::CommentsConfig,
}

// `roots` intentionally stays the opinionated `["tasks"]` rather than collapsing
// to empty: `Config::default()` is called directly (bypassing `parse_config`)
// by a wide swath of test fixtures across the workspace that assume a "tasks"
// root exists. Its value already agrees with the embedded floor
// (`defaults.toml`), so the duplication is harmless — `parse_config` never
// observes this impl (the floor always supplies `roots` first).
impl Default for TasksConfig {
    fn default() -> Self {
        Self {
            roots: vec![RootSpec::Short("tasks".to_owned())],
            derive_tags: DeriveConfig::default(),
            status: Legend::default(),
            priority: PriorityConfig::default(),
            difficulty: NumericField::default(),
            fields: BTreeMap::new(),
            assignee: crate::config_assignee::AssigneeConfig::default(),
            estimate: EstimateConfig::default(),
            tags: TagsConfig::default(),
            body: TaskBodyConfig::default(),
            id: IdConfig::default(),
            naming: crate::naming::NamingConfig::default(),
            links: crate::link::LinksConfig::default(),
            recur: crate::recur::RecurConfig::default(),
            limits: Vec::new(),
            comments: crate::config_comments::CommentsConfig::default(),
        }
    }
}

/// `[tasks.priority]` — the priority numeric field plus its scoring frameworks.
///
/// Carries the same scale and band/anchor classification as `difficulty`, plus
/// framework selection on top. When `framework` names an active framework, a
/// task's priority is computed from that framework's inputs instead of a
/// hand-set integer; `allow-manual` decides whether a raw integer is still
/// accepted as an escape hatch. The `frameworks` table holds the built-in
/// frameworks deep-merged with any user-declared ones.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case")]
pub struct PriorityConfig {
    /// Scale and band classification (shared numeric-field machinery).
    #[serde(flatten)]
    pub field: NumericField,
    /// Active framework name, matched against `frameworks`. `None` = raw numeric.
    pub framework: Option<String>,
    /// Whether a raw integer is accepted while a framework is active.
    pub allow_manual: bool,
    /// Framework registry: built-ins deep-merged with user definitions.
    pub frameworks: BTreeMap<String, Framework>,
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            field: NumericField::default(),
            framework: None,
            allow_manual: true,
            frameworks: BTreeMap::new(),
        }
    }
}

/// `[tasks.id]` — task identity format constraint.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct IdConfig {
    /// Format requirement for an explicit `id:`. Default accepts any non-empty string.
    pub format: IdFormat,
}

/// Format requirement for a task `id:` field.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum IdFormat {
    /// Any non-empty string (the default).
    #[default]
    String,
    /// A UUID v4 (random).
    UuidV4,
    /// A UUID v7 (time-ordered).
    UuidV7,
}

impl IdFormat {
    /// Generate a fresh id in this format, or `None` when identity is the file
    /// stem (the `String` format carries no generator).
    #[must_use]
    pub fn generate(self) -> Option<String> {
        match self {
            Self::String => None,
            Self::UuidV4 => Some(uuid::Uuid::new_v4().to_string()),
            Self::UuidV7 => Some(uuid::Uuid::now_v7().to_string()),
        }
    }
}

impl<'de> Deserialize<'de> for IdFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        match text.as_str() {
            "string" => Ok(Self::String),
            "uuid-v4" => Ok(Self::UuidV4),
            "uuid-v7" => Ok(Self::UuidV7),
            other => Err(serde::de::Error::custom(format!(
                "invalid id format `{other}`; expected `string`, `uuid-v4`, or `uuid-v7`"
            ))),
        }
    }
}

pub use crate::config_actors::{Actor, ActorKind, Role, roles_of};
pub use crate::config_assignee::AssigneeConfig;
pub use crate::config_estimate::{DriftConfig, EstimateConfig};

/// `[tasks.fields.<name>]` — constraints for one configurable core field.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct FieldConfig {
    /// Allowed enum values. Empty = any.
    pub values: Vec<String>,
    /// Default value (informational; not auto-applied during load).
    pub default: Option<String>,
    /// Whether the field must be set.
    pub required: Requirement,
}

/// `[tasks.tags]` — optional tag allow-list.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct TagsConfig {
    /// Allowed tag names. Empty = any.
    pub allowed: Vec<String>,
}

/// Errors from loading or locating a config file.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The config file could not be read.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// The config file contained invalid TOML.
    #[error("parse error: {0}")]
    Parse(#[from] toml::de::Error),
    /// The config parsed but failed a semantic check (e.g. band coverage).
    #[error("invalid config: {0}")]
    Invalid(String),
}

impl Config {
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
        problems.extend(crate::config_required::validate_requirements(self));
        problems.extend(crate::config_links_validate::validate_link_kinds(self));
        problems.extend(crate::config_limits::validate_limits(self));
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
            problems.push(format!("ls.default-view: unknown view `{name}`"));
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems.join("; "))
        }
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
/// # Errors
/// Returns [`ConfigError::Parse`] if the TOML is invalid (the user's text, or,
/// were it ever malformed, the embedded floor itself).
pub fn parse_config(text: &str) -> Result<Config, ConfigError> {
    let mut doc: toml::Table = toml::from_str(EMBEDDED_DEFAULTS)?;
    let overlay: toml::Table = toml::from_str(text)?;
    deep_merge_tables(&mut doc, overlay);
    merge_builtin_frameworks(&mut doc);
    Ok(toml::Value::Table(doc).try_into()?)
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
    crate::config_layers::load_config_layered(root).map(|(config, _sources)| config)
}

/// The conventional config path for a project root.
#[must_use]
pub fn config_path(root: &Path) -> PathBuf {
    root.join(".mindtape").join("config.toml")
}

/// `[tasks.body]` — body-shape constraints.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct TaskBodyConfig {
    /// Section-heading gating rules.
    pub headers: HeadersConfig,
}

pub use crate::body_headers::{HeaderAction, HeaderPathSegment, HeaderRuleConfig, HeadersConfig};

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
