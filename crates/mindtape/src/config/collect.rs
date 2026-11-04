//! `[[tasks.collectors]]`: derived task fields, read as `derived.<name>`.
//!
//! A collector names its field, an optional `when`, a `query` and a `returns`
//! type expression. The collection itself lives in [`crate::collect`]; this
//! module is config shape and load-time checks. Collectors merge by name across
//! layers and packs exactly as the rule arrays do ([`crate::config::packs`]).

use std::path::PathBuf;

use regex::Regex;
use serde::Deserialize;

use crate::body::headers::HeaderPathSegment;
use crate::config::Config;
pub use crate::config::collect_type::{CollectorAtom, CollectorType};
use crate::config::pack_registry::RuleOriginEntry;
use crate::problem::Problem;

/// Where each merged collector was declared, recorded at load.
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
    /// Filter gating the collector; a task it excludes carries no field.
    #[serde(default)]
    #[schemars(extend("x-expression" = "filter"))]
    pub when: Option<String>,
    /// What the collector reads.
    pub query: CollectorQuery,
    /// The type of the value the collector produces.
    pub returns: CollectorType,
    /// The directory a pack-declared collector's `typst` module resolves from.
    /// `None` means the project root. Set at load, never read from config.
    #[serde(skip)]
    pub(crate) root: Option<PathBuf>,
}

impl CollectorSpec {
    /// A collector with no pack root, for a test or a caller building config in
    /// code. Config loading stamps a pack collector's root separately.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        when: Option<String>,
        query: CollectorQuery,
        returns: CollectorType,
    ) -> Self {
        Self {
            name: name.into(),
            when,
            query,
            returns,
            root: None,
        }
    }
}

/// The query a collector runs.
#[derive(Debug, Clone, PartialEq, Deserialize, schemars::JsonSchema)]
#[serde(untagged)]
pub enum CollectorQuery {
    /// A regex over one or more body sections.
    Regex(RegexQuery),
    /// A project Typst function over the whole task or one of its fields.
    Typst(TypstQuery),
}

impl CollectorQuery {
    /// The query's kind, as `mt rules` prints it.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Regex(_) => "regex",
            Self::Typst(_) => "typst",
        }
    }
}

/// A Typst query: call a project function over the canonical task, or over
/// one field of it, and return its value.
#[derive(Debug, Clone, PartialEq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TypstQuery {
    /// Path to the Typst module exporting `function`, relative to the project
    /// root or to the pack directory that declared the collector.
    pub typst: String,
    /// Name of the exported function to call.
    pub function: String,
    /// The field whose value the function receives. Absent means the whole
    /// task, in the shape `mt eval --json` prints.
    #[serde(default)]
    pub field: Option<String>,
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
            CollectorQuery::Regex(query) => validate_regex(query, spec, &path, &mut problems),
            CollectorQuery::Typst(query) => validate_typst(query, &path, &mut problems),
        }
        if let Some(when) = &spec.when {
            if let Some(problem) = crate::filter::expr_problem_in_set(when, &probe, config) {
                problems.push(problem.under(&format!("{path}.when")));
            }
            warn_on_later_reads(when, index, &path, config, &mut problems);
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

fn validate_typst(query: &TypstQuery, path: &str, problems: &mut Vec<Problem>) {
    if query.typst.trim().is_empty() {
        problems.push(Problem::new(format!(
            "{path}.query.typst: must name a Typst module"
        )));
    }
    if query.function.trim().is_empty() {
        problems.push(Problem::new(format!(
            "{path}.query.function: must name a function"
        )));
    }
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

/// A `when` may read only a field an earlier collector produced.
fn warn_on_later_reads(
    when: &str,
    index: usize,
    path: &str,
    config: &Config,
    problems: &mut Vec<Problem>,
) {
    for name in crate::filter::names_in(when, config) {
        let Some(rest) = name.strip_prefix("derived.") else {
            continue;
        };
        let collector = rest.split('.').next().unwrap_or(rest);
        let position = config
            .tasks
            .collectors
            .iter()
            .position(|spec| spec.name == collector);
        if position.is_some_and(|position| position >= index) {
            problems.push(Problem::new(format!(
                "{path}.when: reads `{name}`, which is declared at or after this collector; \
                 collectors run in declaration order"
            )));
        }
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
