//! One descriptor per described field: `[tasks.fields.<name>]`,
//! `[tasks.assignee]` and `[tasks.tags]` each declare allowed values, a
//! requirement and a `values-when` exemption, and validation, the `mt add`
//! slot splice and choice inheritance all read the [`Descriptor`] built here.
//! A task's value is read through [`crate::model::fields::held`].

use typst_harvest::HVal;

use crate::config::tasks::FieldExample;
use crate::config::{Config, Requirement, RequirementRule};
use crate::filter::Filter;
use crate::model::Task;
use crate::problem::Problem;
use crate::suggest::Unknown;

/// How many values a described field takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    /// One string.
    One,
    /// A non-empty list of distinct strings, each allowed.
    Multiple,
    /// A list, empty or repeating allowed; each entry must be allowed.
    Many,
}

impl Cardinality {
    /// Whether the field holds a list.
    #[must_use]
    pub fn is_list(self) -> bool {
        self != Self::One
    }
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
    /// The configured example, as the Typst source a slot shows.
    pub example: Option<String>,
}

/// The `[tasks.tags]` filter default: a closed task keeps a tag dropped from
/// `allowed`, so retiring a tag is one edit rather than a rewrite of every
/// closed task that carried it.
const TAGS_VALUES_WHEN: &str = "!terminal";

/// A list of strings as a Typst array literal.
pub(crate) fn list_source(items: &[String]) -> String {
    let inner: String = items.iter().map(|item| format!("{item:?}, ")).collect();
    format!("({inner})")
}

/// The environment variable an assignee slot reads: the actor running the CLI.
const PRINCIPAL_ENV: &str = "MINDTAPE_PRINCIPAL";

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
            cardinality: if field.multiple {
                Cardinality::Multiple
            } else {
                Cardinality::One
            },
            noun: "value",
            hint: false,
            env: None,
            example: field.example.as_ref().map(|example| match example {
                FieldExample::One(text) => format!("{text:?}"),
                FieldExample::Many(items) => list_source(items),
            }),
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
        example: assignee.example.as_ref().map(|text| format!("{text:?}")),
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
        example: None,
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

    /// The messages for one held value: a value of the wrong cardinality, a
    /// string that is not allowed, and in a list an empty list, a repeated
    /// entry or a non-string entry. A single-valued field without allowed
    /// values takes any shape.
    #[must_use]
    pub fn breaches(&self, task: &Task, value: &HVal) -> Vec<Problem> {
        let refused = |text: &str| self.refusal(task, text);
        match (self.cardinality, value) {
            (Cardinality::One, _) if self.allowed.is_empty() => Vec::new(),
            (Cardinality::One, HVal::Str(text)) if !self.allows(text) => vec![refused(text)],
            (Cardinality::One, HVal::Str(_)) => Vec::new(),
            (Cardinality::One, HVal::Array(_)) => {
                vec!["takes one string, not a list; `multiple = true` declares a list".into()]
            }
            (Cardinality::One, _) => vec!["takes one string".into()],
            (_, HVal::Array(items)) => self.list_breaches(items, refused),
            (_, HVal::Str(_)) => vec!["takes a list of strings, not one string".into()],
            _ => vec!["takes a list of strings".into()],
        }
    }

    fn refusal(&self, task: &Task, text: &str) -> Problem {
        let derived =
            if self.cardinality == Cardinality::Many && !task.tags.iter().any(|tag| tag == text) {
                " derived from folder"
            } else {
                ""
            };
        let problem = Problem::new(format!("`{text}`{derived} is not an allowed {}", self.noun));
        if self.hint {
            problem.with(Unknown::among(
                text,
                self.allowed.iter().map(String::as_str),
            ))
        } else {
            problem
        }
    }

    fn list_breaches(&self, items: &[HVal], refused: impl Fn(&str) -> Problem) -> Vec<Problem> {
        const SHAPE: &str = "takes a list of strings";
        if self.allowed.is_empty() {
            return Vec::new();
        }
        if items.is_empty() {
            return match self.cardinality {
                Cardinality::Many => Vec::new(),
                _ => vec!["an empty list names no allowed value".into()],
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
                    out.push(format!("`{text}` is listed twice").into());
                }
                HVal::Str(text) => seen.push(text),
                _ if !out.iter().any(|message| message == SHAPE) => out.push(SHAPE.into()),
                _ => {}
            }
        }
        out
    }
}
