//! One descriptor per described field: `[tasks.fields.<name>]`,
//! `[tasks.assignee]` and `[tasks.tags]` each declare allowed values, a
//! requirement and a `values-when` exemption, and validation, the `mt add`
//! slot splice and choice inheritance all read the [`Descriptor`] built here.
//! A task's value is read through [`crate::fields::held`].

use typst_harvest::HVal;

use crate::config::{Config, Requirement, RequirementRule};
use crate::filter::Filter;
use crate::model::Task;

/// How many values a described field takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    /// One string.
    One,
    /// One string or a list of distinct strings, each allowed.
    Any,
    /// A list, empty or repeating allowed; each entry must be allowed.
    Many,
}

/// One field's rules, whichever config table declared them: `[tasks.fields]`,
/// `[tasks.assignee]` or `[tasks.tags]`. Validation, the `mt add` slot splice
/// and choice inheritance all read it, so a field's allowed values, default,
/// requirement and exemption live in one place.
#[derive(Debug, Clone)]
pub struct Descriptor<'a> {
    /// The field's name: `kind`, `assignee`, `tags` or an `x-<ns>` name.
    pub name: &'a str,
    /// The config table that declares it, as the fill note names it.
    pub path: String,
    /// Allowed values. Empty accepts any value.
    pub allowed: &'a [String],
    /// The field's `required` rule, compiled.
    pub must: RequirementRule,
    /// The `required` rule as written.
    pub required: &'a Requirement,
    /// What `mt add` fills in for a required field nobody set.
    pub default: Option<&'a str>,
    /// The tasks held to `allowed`, `None` for every task.
    pub when: Option<Filter>,
    /// How many values the field takes.
    pub cardinality: Cardinality,
    /// What an allowed value is called in a message.
    pub noun: &'static str,
    /// Whether a rejected value names the nearest allowed one.
    pub hint: bool,
    /// The variable a slot for the field reads first.
    pub env: Option<&'static str>,
    /// The example a slot shows when nothing else does.
    pub example: String,
}

/// The `[tasks.tags]` filter default: a closed task keeps a tag dropped from
/// `allowed`, so retiring a tag is one edit rather than a rewrite of every
/// closed task that carried it.
const TAGS_VALUES_WHEN: &str = "!terminal";

/// The environment variable an assignee slot reads: the actor running the CLI.
const PRINCIPAL_ENV: &str = "MINDTAPE_PRINCIPAL";

/// The example an assignee slot shows when the project names no actor.
const PLACEHOLDER_ASSIGNEE: &str = "someone";

/// Every described field of `config`: each `[tasks.fields]` entry, then
/// `assignee`, then `tags`.
#[must_use]
pub fn descriptors(config: &Config) -> Vec<Descriptor<'_>> {
    let tasks = &config.tasks;
    let when = |source: Option<&str>| source.and_then(|source| Filter::parse(source).ok());
    let mut out: Vec<Descriptor<'_>> = tasks
        .fields
        .iter()
        .map(|(name, field)| Descriptor {
            name,
            path: format!("[tasks.fields.{name}]"),
            allowed: &field.values,
            must: RequirementRule::compile(&field.required),
            required: &field.required,
            default: field.default.as_deref(),
            when: when(field.values_when.as_deref()),
            cardinality: if name == "kind" {
                Cardinality::One
            } else {
                Cardinality::Any
            },
            noun: "value",
            hint: false,
            env: None,
            example: name.strip_prefix("x-").unwrap_or(name).to_owned(),
        })
        .collect();
    let assignee = &tasks.assignee;
    out.push(Descriptor {
        name: "assignee",
        path: "[tasks.assignee]".to_owned(),
        allowed: &assignee.allowed,
        must: RequirementRule::compile(&assignee.required),
        required: &assignee.required,
        default: None,
        when: when(assignee.values_when.as_deref()),
        cardinality: Cardinality::One,
        noun: "assignee",
        hint: false,
        env: Some(PRINCIPAL_ENV),
        example: config
            .actors
            .first()
            .map_or(PLACEHOLDER_ASSIGNEE, |actor| actor.name.as_str())
            .to_owned(),
    });
    out.push(Descriptor {
        name: "tags",
        path: "[tasks.tags]".to_owned(),
        allowed: &tasks.tags.allowed,
        must: RequirementRule::Fixed(false),
        required: &Requirement::No,
        default: None,
        when: when(Some(
            tasks
                .tags
                .values_when
                .as_deref()
                .unwrap_or(TAGS_VALUES_WHEN),
        )),
        cardinality: Cardinality::Many,
        noun: "tag",
        hint: true,
        env: None,
        example: "tag".to_owned(),
    });
    out
}

impl Descriptor<'_> {
    /// Whether `task` is held to the allowed values.
    #[must_use]
    pub fn holds(&self, task: &Task, config: &Config) -> bool {
        self.when
            .as_ref()
            .is_none_or(|filter| filter.matches(task, config).unwrap_or(true))
    }

    /// Whether `value` is one of the allowed values. Always, when none are
    /// declared.
    #[must_use]
    pub fn allows(&self, value: &str) -> bool {
        self.allowed.is_empty() || self.allowed.iter().any(|allowed| allowed == value)
    }

    /// The messages for one held value: each string must be allowed, and a
    /// list of the wrong cardinality, an empty or repeating one, or a
    /// non-string entry is refused. A field without allowed values takes any
    /// shape.
    #[must_use]
    pub fn breaches(&self, task: &Task, value: &HVal) -> Vec<String> {
        const SHAPE: &str = "a field with allowed values takes a string or a list of strings";
        if self.allowed.is_empty() {
            return Vec::new();
        }
        let refused = |text: &str| {
            let derived = if self.cardinality == Cardinality::Many
                && !task.tags.iter().any(|tag| tag == text)
            {
                " derived from folder"
            } else {
                ""
            };
            let hint = if self.hint {
                crate::suggest::hint(self.noun, text, self.allowed)
            } else {
                String::new()
            };
            format!("`{text}`{derived} is not an allowed {}{hint}", self.noun)
        };
        match value {
            HVal::Str(text) if !self.allows(text) => vec![refused(text)],
            HVal::Str(_) => Vec::new(),
            HVal::Array(items) if self.cardinality != Cardinality::One => {
                self.list_breaches(items, SHAPE, refused)
            }
            _ => vec![SHAPE.to_owned()],
        }
    }

    fn list_breaches(
        &self,
        items: &[HVal],
        shape: &str,
        refused: impl Fn(&str) -> String,
    ) -> Vec<String> {
        if items.is_empty() {
            return match self.cardinality {
                Cardinality::Many => Vec::new(),
                _ => vec!["an empty list names no allowed value".to_owned()],
            };
        }
        let mut out = Vec::new();
        let mut seen: Vec<&str> = Vec::new();
        for item in items {
            match item {
                HVal::Str(text) if !self.allows(text) => out.push(refused(text)),
                HVal::Str(text)
                    if self.cardinality != Cardinality::Many && seen.contains(&text.as_str()) =>
                {
                    out.push(format!("`{text}` is listed twice"));
                }
                HVal::Str(text) => seen.push(text),
                _ if !out.iter().any(|message| message == shape) => out.push(shape.to_owned()),
                _ => {}
            }
        }
        out
    }
}
