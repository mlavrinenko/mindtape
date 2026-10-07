//! `[tasks]` config: the section struct itself, plus the small field types it
//! owns directly and that never grew big enough to earn their own module —
//! priority (numeric field + scoring framework selection), task id format,
//! ad-hoc field constraints, the tag allow-list, and body-shape config.
//!
//! Field types that already outgrew this file live in their own sibling
//! instead and are only referenced here: roots/derive-tags (`config/roots.rs`),
//! the status legend (`config/status.rs`), assignee (`config/assignee.rs`),
//! estimate (`config/estimate.rs`), naming (`crate::model::naming`), links
//! (`crate::link`), recur (`crate::recur`), rules (`rules/mod.rs`), and
//! comments (`config/comments.rs`).

use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Deserializer};

use crate::body::content::ContentConfig;
use crate::body::headers::HeadersConfig;
use crate::config::dicts::Dicts;
use crate::config::estimate::EstimateConfig;
use crate::config::required::Requirement;
use crate::config::roots::{DeriveConfig, RootSpec};
use crate::config::status::Legend;
use crate::framework::Framework;
use crate::model::numeric::{Anchor, Band, NumericField, ScaleKind};

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
    /// The priority scale, its bands, and an optional scoring framework that
    /// computes priority from inputs.
    pub priority: PriorityConfig,
    /// The difficulty scale and its labelled anchors or bands.
    pub difficulty: NumericField,
    /// Allowed values, default and requirement for `kind` and for each
    /// `x-<ns>` extension field, keyed by field name. Any other key fails
    /// config load.
    pub fields: BTreeMap<String, FieldConfig>,
    /// Which actors `assignee:` may name, and whether it is required.
    pub assignee: crate::config::assignee::AssigneeConfig,
    /// Allowed estimate units, and whether an estimate is required.
    pub estimate: EstimateConfig,
    /// Tag allow-list, checked on open tasks only.
    pub tags: TagsConfig,
    /// Rules over a task body's headings and section content.
    pub body: TaskBodyConfig,
    /// Task id format.
    pub id: IdConfig,
    /// The filename stem template `mt add` and `mt recur` use, and what
    /// `mt add` does when the stem is taken (`dedup`).
    pub naming: crate::model::naming::NamingConfig,
    /// Custom link kinds that `links:` edges may use, beside the built-in ones.
    pub links: crate::link::LinksConfig,
    /// Whether recurring tasks are enabled, and the units, anchors and limits
    /// `mt recur` accepts.
    pub recur: crate::recur::RecurConfig,
    /// Assertions every selected task must satisfy. `mt check` enforces them,
    /// and so does every write command before it writes.
    pub rules: Vec<crate::rules::RuleSpec>,
    /// Review-comment policy over a task's whole source file (`mt check`).
    pub comments: crate::config::comments::CommentsConfig,
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
            assignee: crate::config::assignee::AssigneeConfig::default(),
            estimate: EstimateConfig::default(),
            tags: TagsConfig::default(),
            body: TaskBodyConfig::default(),
            id: IdConfig::default(),
            naming: crate::model::naming::NamingConfig::default(),
            links: crate::link::LinksConfig::default(),
            recur: crate::recur::RecurConfig::default(),
            rules: Vec::new(),
            comments: crate::config::comments::CommentsConfig::default(),
        }
    }
}

/// `[tasks.priority]` — the priority scale and its scoring frameworks.
///
/// It takes the same scale, bands and anchors as `[tasks.difficulty]`. When
/// `framework` names a framework, `mt` computes a task's priority from that
/// framework's inputs. `allow-manual` decides whether a raw integer is still
/// accepted. `frameworks` holds the built-in frameworks merged with any the
/// project declares.
// `Deserialize` is hand-written: serde cannot combine `flatten` with
// `deny_unknown_fields`, so `PriorityRaw` spells the scale keys out.
#[derive(Debug, Clone, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PriorityConfig {
    /// Scale and band classification (shared numeric-field machinery).
    #[serde(flatten)]
    pub field: NumericField,
    /// Name of the framework that computes priority, a key of `frameworks`.
    /// Absent means priority is a raw number.
    pub framework: Option<String>,
    /// Whether a raw integer is accepted while a framework is active.
    pub allow_manual: bool,
    /// Scoring frameworks by name: the built-in ones merged with any declared
    /// here.
    pub frameworks: BTreeMap<String, Framework>,
}

/// The keys of `[tasks.priority]` with the scale keys inlined, so an unknown
/// key is an error.
#[derive(Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
struct PriorityRaw {
    kind: ScaleKind,
    min: f64,
    max: f64,
    bands: Vec<Band>,
    anchors: Vec<Anchor>,
    required: Requirement,
    example: Option<f64>,
    framework: Option<String>,
    allow_manual: bool,
    frameworks: BTreeMap<String, Framework>,
}

impl Default for PriorityRaw {
    fn default() -> Self {
        let PriorityConfig {
            field,
            framework,
            allow_manual,
            frameworks,
        } = PriorityConfig::default();
        Self {
            kind: field.kind,
            min: field.min,
            max: field.max,
            bands: field.bands,
            anchors: field.anchors,
            required: field.required,
            example: field.example,
            framework,
            allow_manual,
            frameworks,
        }
    }
}

impl<'de> Deserialize<'de> for PriorityConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = PriorityRaw::deserialize(deserializer)?;
        Ok(Self {
            field: NumericField {
                kind: raw.kind,
                min: raw.min,
                max: raw.max,
                bands: raw.bands,
                anchors: raw.anchors,
                required: raw.required,
                example: raw.example,
            },
            framework: raw.framework,
            allow_manual: raw.allow_manual,
            frameworks: raw.frameworks,
        })
    }
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
    /// Format an explicit `id:` must have. The default, `string`, accepts any
    /// non-empty string.
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

/// `[tasks.fields.<name>]` — allowed values, default and requirement for
/// `kind` or one `x-<ns>` extension field.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct FieldConfig {
    /// Allowed values. Empty accepts any value, and a non-empty list takes a
    /// string, or a list of distinct strings for an `x-<ns>` field. An entry `@<dict>` stands for every entry of
    /// `[dicts.<dict>]`, so the list may mix literal values and dictionaries.
    // `expand_dict_refs` expands the `@<dict>` entries at config load.
    pub values: Vec<String>,
    /// Value `mt add` fills in when the field is required and neither the
    /// template nor the caller sets it. `mt check` never applies it.
    pub default: Option<String>,
    /// Whether the field must be set.
    pub required: Requirement,
    /// A filter naming the tasks held to `values`; every other task keeps
    /// whatever it carries. Absent holds every task.
    pub values_when: Option<String>,
    /// Whether an `x-<ns>` field holds a list of strings rather than one
    /// string. `mt check` refuses a string in such a field, and a list in a
    /// field with `values` that does not declare it. `mt add` offers the field
    /// as a multi-choice slot.
    pub multiple: bool,
    /// The value `mt check` files a spliced slot for this field with: a string,
    /// or a list of strings for a `multiple` field. A required field with no
    /// `example` and no `default` leaves that slot unfillable, so
    /// `mt check` names this table.
    pub example: Option<FieldExample>,
}

/// A `[tasks.fields.<name>] example`: one string, or a list for a `multiple`
/// field.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(untagged)]
pub enum FieldExample {
    /// The example of a single-valued field.
    One(String),
    /// The example of a `multiple` field.
    Many(Vec<String>),
}

/// `[tasks.tags]` — optional tag allow-list.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TagsConfig {
    /// Allowed tag names. Empty accepts any tag. The loader derives folder tags
    /// again on every load, so a moved file gets its new folder's tags. An entry
    /// `@<dict>` stands for every entry of `[dicts.<dict>]`.
    // `expand_dict_refs` expands the `@<dict>` entries at config load.
    pub allowed: Vec<String>,
    /// A filter naming the tasks held to `allowed`. Absent means `!terminal`,
    /// so a closed task keeps a tag dropped from the list.
    pub values_when: Option<String>,
}

/// Expand every `@<dict>` reference in `[tasks.tags] allowed` and each
/// `[tasks.fields.<name>] values` against `dicts`, in place. Run once, at
/// config load ([`crate::config::layers::build_config`]), after `[dicts]`
/// itself has compiled — never per task — so every downstream reader
/// (validation, `mt add` slot completion, the generated schema) sees a plain,
/// already-resolved list and needs no change of its own.
///
/// A `@`-prefixed entry always names a dictionary — the same sigil `mt ls`
/// reads as a view reference — so there is no escape for a literal tag or
/// field value that itself starts with `@`: an unresolvable one fails the
/// load exactly like a typo'd dict name (see [`expand_list`]). A list may mix
/// literal entries and `@<dict>` references freely; the result is their
/// union in declaration order, deduplicated (first occurrence wins) — a
/// referenced dictionary contributes its own entries in the dictionary's own
/// (alphabetical) order, since [`Dicts`] carries no declaration order of its
/// own to preserve.
///
/// # Errors
/// One message per unknown dictionary name referenced, each carrying a
/// nearest-match suggestion, joined with `; `.
pub(crate) fn expand_dict_refs(tasks: &mut TasksConfig, dicts: &Dicts) -> Result<(), String> {
    let mut problems = Vec::new();
    if let Err(msg) = expand_list(&mut tasks.tags.allowed, dicts, "tasks.tags.allowed") {
        problems.push(msg);
    }
    for (name, field) in &mut tasks.fields {
        let label = format!("tasks.fields.{name}.values");
        if let Err(msg) = expand_list(&mut field.values, dicts, &label) {
            problems.push(msg);
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

/// Expand one `values`/`allowed` list in place against `dicts`: a `@<dict>`
/// entry is replaced by that dictionary's entries, a literal entry is kept
/// as-is, and the whole result is deduplicated (first occurrence wins).
/// `label` names the list in an error message (`tasks.tags.allowed`,
/// `tasks.fields.kind.values`).
fn expand_list(list: &mut Vec<String>, dicts: &Dicts, label: &str) -> Result<(), String> {
    let mut out = Vec::with_capacity(list.len());
    let mut seen: HashSet<String> = HashSet::new();
    let mut problems = Vec::new();
    for entry in list.iter() {
        let Some(name) = entry.strip_prefix('@') else {
            if seen.insert(entry.clone()) {
                out.push(entry.clone());
            }
            continue;
        };
        let Some(dict) = dicts.get(name) else {
            let known: Vec<String> = dicts.names().map(ToOwned::to_owned).collect();
            problems.push(format!(
                "{label}: unknown dict `{name}`{}",
                crate::suggest::hint("dict", name, &known)
            ));
            continue;
        };
        for value in dict.entries() {
            if seen.insert(value.to_owned()) {
                out.push(value.to_owned());
            }
        }
    }
    if problems.is_empty() {
        *list = out;
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

/// Core fields that carry their own `[tasks.<name>]` table, which a
/// `[tasks.fields.<name>]` entry would shadow without effect.
const OWN_TABLE_FIELDS: [&str; 9] = [
    "status",
    "priority",
    "difficulty",
    "assignee",
    "estimate",
    "tags",
    "id",
    "links",
    "recur",
];

/// Load-time check of every `[tasks.fields.<name>]` key: only `kind` and
/// `x-<ns>` names are read by validation, so any other key is refused instead
/// of loading as an inert declaration. `kind` holds one string, so it takes no
/// `multiple`.
pub(crate) fn validate_field_names(tasks: &TasksConfig) -> Vec<String> {
    let mut out: Vec<String> = tasks
        .fields
        .keys()
        .filter(|name| *name != "kind" && !name.starts_with("x-"))
        .map(|name| {
            if OWN_TABLE_FIELDS.contains(&name.as_str()) {
                format!("tasks.fields.{name}: `{name}` is configured in [tasks.{name}]")
            } else {
                format!("tasks.fields.{name}: unknown field; [tasks.fields] takes `kind` and `x-<ns>` names")
            }
        })
        .collect();
    out.extend(tasks.fields.iter().filter_map(|(name, field)| {
        let many = matches!(field.example, Some(FieldExample::Many(_)));
        let one = matches!(field.example, Some(FieldExample::One(_)));
        ((many && !field.multiple) || (one && field.multiple)).then(|| {
            let want = if field.multiple { "a list" } else { "a string" };
            format!("tasks.fields.{name}.example: the field takes {want}")
        })
    }));
    out.extend(
        tasks
            .fields
            .iter()
            .filter(|(name, field)| field.multiple && !name.starts_with("x-"))
            .map(|(name, _)| {
                format!("tasks.fields.{name}.multiple: only an `x-<ns>` field holds a list")
            }),
    );
    out
}

/// `[tasks.body]` — body-shape constraints.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct TaskBodyConfig {
    /// Section-heading gating rules.
    pub headers: HeadersConfig,
    /// Section-content gating rules.
    pub content: ContentConfig,
}

#[cfg(test)]
#[path = "tasks_tests.rs"]
mod tests;
