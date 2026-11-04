//! `[[tasks.collectors]]`: derived task fields, declared as a query over the
//! task and read as `derived.<name>`.
//!
//! A collector names its derived field, an optional `when` filter gating it,
//! a `query` and a `returns` type expression. The regex query reads a body
//! section and returns a list of strings or paths. The collection itself lives
//! in [`crate::collect`]; this module is config shape and load-time checks.
//!
//! Collectors merge by name across layers and packs exactly as the rule arrays
//! do ([`crate::config::packs`]), so a project overrides a shipped collector by
//! redeclaring its name or drops it with `disable`.

use std::collections::BTreeMap;
use std::fmt;

use regex::Regex;
use serde::{Deserialize, Deserializer};

use crate::body::headers::HeaderPathSegment;
use crate::config::Config;
use crate::config::pack_registry::RuleOriginEntry;
use crate::problem::Problem;

/// Where each merged collector was declared, recorded by
/// [`crate::config::layers::build_config`]. Derived, never declared, never
/// equal-compared — the same reading [`crate::rules::sites::RuleSites`] takes.
#[derive(Debug, Clone, Default)]
pub(crate) struct CollectorOrigins(pub(crate) Vec<RuleOriginEntry>);

impl PartialEq for CollectorOrigins {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// One `[[tasks.collectors]]` entry.
#[derive(Debug, Clone, PartialEq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CollectorSpec {
    /// The derived field's name: the collector is read as `derived.<name>`.
    pub name: String,
    /// Filter expression gating which tasks the collector runs on. Absent
    /// means every task. A task the filter excludes carries no field.
    #[serde(default)]
    #[schemars(extend("x-expression" = "filter"))]
    pub when: Option<String>,
    /// What the collector reads.
    pub query: CollectorQuery,
    /// The type of the value the collector produces.
    pub returns: CollectorType,
}

/// The query a collector runs.
#[derive(Debug, Clone, PartialEq, Deserialize, schemars::JsonSchema)]
#[serde(untagged)]
pub enum CollectorQuery {
    /// A regex over one or more body sections.
    Regex(RegexQuery),
}

impl CollectorQuery {
    /// The query's kind, as `mt rules` prints it.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Regex(_) => "regex",
        }
    }
}

/// A regex query: the first capture group (or the whole match) of every match
/// in every selected section's content.
#[derive(Debug, Clone, PartialEq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RegexQuery {
    /// The body section to read: a heading path, or one heading name.
    pub section: SectionSpec,
    /// The regex to run over each selected section's content.
    pub pattern: String,
    /// The capture group to take, counting from 0. Absent takes the first
    /// capture group when the pattern has one, else the whole match.
    #[serde(default)]
    pub group: Option<usize>,
}

/// A body section selector: one heading name, or a path through the heading
/// tree.
#[derive(Debug, Clone, PartialEq, Deserialize, schemars::JsonSchema)]
#[serde(untagged)]
pub enum SectionSpec {
    /// One exact heading name, matched as a direct child of the body root.
    Name(String),
    /// A path through the heading tree, each step a heading segment.
    Path(Vec<HeaderPathSegment>),
}

impl SectionSpec {
    /// The heading path to select.
    #[must_use]
    pub fn path(&self) -> Vec<HeaderPathSegment> {
        match self {
            Self::Name(name) => vec![HeaderPathSegment::Exact(name.clone())],
            Self::Path(path) => path.clone(),
        }
    }
}

/// A `returns` type expression: an atom, a one-element list `[T]`, or a record
/// `{ field = T }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectorType {
    /// A scalar type.
    Atom(CollectorAtom),
    /// A list of one type, written `[T]`.
    List(Box<CollectorType>),
    /// A record of named fields, written `{ field = T }`.
    Record(BTreeMap<String, CollectorType>),
}

/// One atom of a [`CollectorType`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectorAtom {
    /// `true` or `false`.
    Bool,
    /// A whole number.
    Int,
    /// A whole or fractional number.
    Number,
    /// Text.
    String,
    /// A path, read as text but counted by `count_paths()`.
    Path,
    /// A `YYYY-MM-DD` date.
    Date,
}

impl CollectorAtom {
    /// Parse one atom name.
    ///
    /// # Errors
    /// A message naming the unknown atom and the known ones.
    pub fn parse(text: &str) -> Result<Self, String> {
        match text {
            "bool" => Ok(Self::Bool),
            "int" => Ok(Self::Int),
            "number" => Ok(Self::Number),
            "string" => Ok(Self::String),
            "path" => Ok(Self::Path),
            "date" => Ok(Self::Date),
            other => Err(format!(
                "unknown type: {}; use bool, int, number, string, path or date",
                crate::config::quote(other)
            )),
        }
    }

    /// The atom as a config spells it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Number => "number",
            Self::String => "string",
            Self::Path => "path",
            Self::Date => "date",
        }
    }
}

impl CollectorType {
    /// The type as a config spells it, for `mt rules`.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Atom(atom) => atom.as_str().to_owned(),
            Self::List(inner) => format!("[{}]", inner.describe()),
            Self::Record(fields) => {
                let fields: Vec<String> = fields
                    .iter()
                    .map(|(name, ty)| format!("{name} = {}", ty.describe()))
                    .collect();
                format!("{{ {} }}", fields.join(", "))
            }
        }
    }

    /// Whether this is a scalar, which a filter may compare and `--sort` may
    /// order.
    #[must_use]
    pub fn is_scalar(&self) -> bool {
        matches!(self, Self::Atom(_))
    }

    /// The type of one record field, or `None` for a non-record or an unknown
    /// field.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&Self> {
        match self {
            Self::Record(fields) => fields.get(name),
            _ => None,
        }
    }

    /// Whether this is the output of a regex query: `["string"]` or
    /// `["path"]`.
    #[must_use]
    pub fn is_regex_output(&self) -> bool {
        matches!(
            self,
            Self::List(inner)
                if matches!(
                    inner.as_ref(),
                    Self::Atom(CollectorAtom::String | CollectorAtom::Path)
                )
        )
    }

    /// Whether this is a list of paths, which `count_paths()` reads.
    #[must_use]
    pub fn is_path_list(&self) -> bool {
        matches!(
            self,
            Self::List(inner) if matches!(inner.as_ref(), Self::Atom(CollectorAtom::Path))
        )
    }

    /// Whether `value` matches this type.
    #[must_use]
    pub fn accepts(&self, value: &typst_harvest::HVal) -> bool {
        use typst_harvest::HVal;
        match self {
            Self::Atom(atom) => match (atom, value) {
                (CollectorAtom::Bool, HVal::Bool(_))
                | (CollectorAtom::Int, HVal::Int(_))
                | (CollectorAtom::Number, HVal::Int(_) | HVal::Float(_))
                | (CollectorAtom::String | CollectorAtom::Path, HVal::Str(_))
                | (CollectorAtom::Date, HVal::Date(_)) => true,
                (CollectorAtom::Date, HVal::Str(text)) => crate::date::parse_ymd(text).is_some(),
                _ => false,
            },
            Self::List(inner) => match value {
                HVal::Array(items) => items.iter().all(|item| inner.accepts(item)),
                _ => false,
            },
            Self::Record(fields) => match value {
                HVal::Dict(entries) => {
                    entries.len() == fields.len()
                        && fields.iter().all(|(name, ty)| {
                            entries.get(name).is_some_and(|value| ty.accepts(value))
                        })
                }
                _ => false,
            },
        }
    }
}

impl<'de> Deserialize<'de> for CollectorType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TypeVisitor;

        impl<'de> serde::de::Visitor<'de> for TypeVisitor {
            type Value = CollectorType;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a type name, a one-element array `[T]`, or a record `{ field = T }`")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                CollectorAtom::parse(value)
                    .map(CollectorType::Atom)
                    .map_err(serde::de::Error::custom)
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let inner: Option<CollectorType> = seq.next_element()?;
                let Some(inner) = inner else {
                    return Err(serde::de::Error::custom(
                        "a list type takes exactly one element: [T]",
                    ));
                };
                if seq.next_element::<CollectorType>()?.is_some() {
                    return Err(serde::de::Error::custom(
                        "a list type takes exactly one element: [T]",
                    ));
                }
                Ok(CollectorType::List(Box::new(inner)))
            }

            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                let mut fields = BTreeMap::new();
                while let Some((name, ty)) = map.next_entry::<String, CollectorType>()? {
                    fields.insert(name, ty);
                }
                Ok(CollectorType::Record(fields))
            }
        }

        deserializer.deserialize_any(TypeVisitor)
    }
}

impl schemars::JsonSchema for CollectorType {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "CollectorType".into()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        "mindtape::config::collect::CollectorType".into()
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "description": "A `returns` type expression: one of `bool`, `int`, \
                `number`, `string`, `path` and `date`; a one-element array `[T]` \
                for a list; or a table `{ field = T }` for a record.",
            "oneOf": [
                {
                    "type": "string",
                    "enum": ["bool", "int", "number", "string", "path", "date"]
                },
                {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": 1,
                    "description": "A list of one type, written `[T]`."
                },
                {
                    "type": "object",
                    "description": "A record of named fields, written `{ field = T }`.",
                    "additionalProperties": true
                }
            ]
        })
    }
}

impl fmt::Display for CollectorType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.describe())
    }
}

/// The type expression `mt rules` prints for one collector.
#[must_use]
pub fn returns_of(spec: &CollectorSpec) -> String {
    spec.returns.describe()
}

/// Load-time checks for `[[tasks.collectors]]`: a usable name, a query that
/// compiles, a `returns` the query can produce, and no two entries sharing a
/// name within one source. The pack merge already collapsed cross-source
/// duplicates.
#[must_use]
pub(crate) fn validate_collectors(config: &Config) -> Vec<Problem> {
    let probe = crate::model::Task::default();
    let mut problems = Vec::new();
    for (index, spec) in config.tasks.collectors.iter().enumerate() {
        let path = format!("tasks.collectors[{index}]");
        if !valid_name(&spec.name) {
            problems.push(Problem::new(format!(
                "{path}.name: {} is not a field name; use letters, digits, `-` and `_`, \
                 starting with a letter or `_`",
                crate::config::quote(&spec.name)
            )));
        }
        match &spec.query {
            CollectorQuery::Regex(query) => {
                validate_regex(query, spec, &path, &mut problems);
            }
        }
        if let Some(when) = &spec.when
            && let Some(problem) = crate::filter::expr_problem_in_set(when, &probe, config)
        {
            problems.push(problem.under(&format!("{path}.when")));
        }
    }
    problems.extend(
        crate::body::headers::duplicate_rule_names(
            config
                .tasks
                .collectors
                .iter()
                .map(|spec| Some(spec.name.as_str())),
            "tasks.collectors",
        )
        .into_iter()
        .map(Problem::from),
    );
    problems
}

fn validate_regex(
    query: &RegexQuery,
    spec: &CollectorSpec,
    path: &str,
    problems: &mut Vec<Problem>,
) {
    let section = query.section.path();
    if section.is_empty() {
        problems.push(Problem::new(format!(
            "{path}.query.section: must name a section"
        )));
    } else if let Err(err) = crate::body::gate::compile_path(&section) {
        problems.push(Problem::new(format!("{path}.query.section: {err}")));
    }
    let compiled = match Regex::new(&query.pattern) {
        Ok(compiled) => compiled,
        Err(err) => {
            problems.push(Problem::new(format!(
                "{path}.query.pattern: invalid regex: {}",
                last_line(&err.to_string())
            )));
            return;
        }
    };
    if let Some(group) = query.group
        && group >= compiled.captures_len()
    {
        problems.push(Problem::new(format!(
            "{path}.query.group: the pattern has {}, so {group} does not exist",
            crate::count(compiled.captures_len().saturating_sub(1), "capture group")
        )));
    }
    if !spec.returns.is_regex_output() {
        problems.push(Problem::new(format!(
            "{path}.returns: a regex query returns [\"string\"] or [\"path\"], not {}",
            spec.returns.describe()
        )));
    }
}

/// Whether `name` is a usable derived-field name: a letter or `_`, then
/// letters, digits, `-` and `_`. A `.` would make a record read ambiguous.
fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_alphanumeric() || ch == '-' || ch == '_')
}

/// The last line of a `regex` error, without its `error: ` label.
fn last_line(text: &str) -> String {
    let line = text
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(text)
        .trim();
    line.strip_prefix("error: ").unwrap_or(line).to_owned()
}

#[cfg(test)]
#[path = "collect_tests.rs"]
mod tests;
