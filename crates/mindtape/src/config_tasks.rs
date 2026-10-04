//! `[tasks]` config: the section struct itself, plus the small field types it
//! owns directly and that never grew big enough to earn their own module —
//! priority (numeric field + scoring framework selection), task id format,
//! ad-hoc field constraints, the tag allow-list, and body-shape config.
//!
//! Field types that already outgrew this file live in their own sibling
//! instead and are only referenced here: roots/derive-tags (`config_roots`),
//! the status legend (`config_status`), assignee (`config_assignee`),
//! estimate (`config_estimate`), naming (`crate::naming`), links
//! (`crate::link`), recur (`crate::recur`), rules (`config_rules`), and
//! comments (`config_comments`).

use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Deserializer};

use crate::body_content::ContentConfig;
use crate::body_headers::HeadersConfig;
use crate::config_dicts::Dicts;
use crate::config_estimate::EstimateConfig;
use crate::config_required::Requirement;
use crate::config_roots::{DeriveConfig, RootSpec};
use crate::config_status::Legend;
use crate::framework::Framework;
use crate::numeric::{Anchor, Band, NumericField, ScaleKind};

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
    pub assignee: crate::config_assignee::AssigneeConfig,
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
    pub naming: crate::naming::NamingConfig,
    /// Custom link kinds that `links:` edges may use, beside the built-in ones.
    pub links: crate::link::LinksConfig,
    /// Whether recurring tasks are enabled, and the units, anchors and limits
    /// `mt recur` accepts.
    pub recur: crate::recur::RecurConfig,
    /// Assertions every selected task must satisfy. `mt check` enforces them,
    /// and so does every write command before it writes.
    pub rules: Vec<crate::config_rules::RuleSpec>,
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
            rules: Vec::new(),
            comments: crate::config_comments::CommentsConfig::default(),
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
/// config load ([`crate::config_layers::build_config`]), after `[dicts]`
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
/// of loading as an inert declaration.
pub(crate) fn validate_field_names(tasks: &TasksConfig) -> Vec<String> {
    tasks
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
        .collect()
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
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use std::collections::BTreeMap;

    use super::{FieldConfig, TagsConfig, TasksConfig, expand_dict_refs};
    use crate::config_dicts::{DictSpec, Dicts, compile_dicts_rootless};

    /// Compile a `Dicts` from `(name, values)` pairs — inline `values` only,
    /// which needs no project root.
    fn dicts(spec: &[(&str, &[&str])]) -> Dicts {
        let map: BTreeMap<String, DictSpec> = spec
            .iter()
            .map(|(name, values)| {
                let spec = DictSpec {
                    values: values.iter().map(|v| (*v).to_owned()).collect(),
                    ..DictSpec::default()
                };
                ((*name).to_owned(), spec)
            })
            .collect();
        compile_dicts_rootless(&map).expect("inline-only dicts always compile")
    }

    /// No `@` entry anywhere leaves both lists exactly as declared.
    #[test]
    fn a_list_with_no_at_refs_is_untouched() {
        let mut tasks = TasksConfig {
            tags: TagsConfig {
                allowed: vec!["swept".to_owned(), "size".to_owned()],
                ..TagsConfig::default()
            },
            ..TasksConfig::default()
        };
        expand_dict_refs(&mut tasks, &dicts(&[])).unwrap();
        assert_eq!(tasks.tags.allowed, ["swept", "size"]);
    }

    /// `[tasks.tags] allowed`'s expansion applies equally to a
    /// `[tasks.fields.<name>] values` list.
    #[test]
    fn a_dict_ref_in_a_field_s_values_expands() {
        let mut tasks = TasksConfig::default();
        tasks.fields.insert(
            "kind".to_owned(),
            FieldConfig {
                values: vec!["@kinds".to_owned()],
                ..FieldConfig::default()
            },
        );
        expand_dict_refs(&mut tasks, &dicts(&[("kinds", &["bug", "chore"])])).unwrap();
        assert_eq!(tasks.fields["kind"].values, ["bug", "chore"]);
    }

    /// An unknown ref in `tags.allowed` and one in a field's `values` are
    /// both reported, joined into the one returned message.
    #[test]
    fn unknown_refs_in_both_lists_are_joined_into_one_error() {
        let mut tasks = TasksConfig {
            tags: TagsConfig {
                allowed: vec!["@ghost".to_owned()],
                ..TagsConfig::default()
            },
            ..TasksConfig::default()
        };
        tasks.fields.insert(
            "kind".to_owned(),
            FieldConfig {
                values: vec!["@phantom".to_owned()],
                ..FieldConfig::default()
            },
        );
        let err = expand_dict_refs(&mut tasks, &dicts(&[])).unwrap_err();
        assert!(
            err.contains("tasks.tags.allowed: unknown dict `ghost`"),
            "{err}"
        );
        assert!(
            err.contains("tasks.fields.kind.values: unknown dict `phantom`"),
            "{err}"
        );
    }
}
