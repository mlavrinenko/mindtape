//! `[[tasks.collectors]]`: derived task fields, read as `derived.<name>`.
//!
//! A collector names its field, an optional `when`, a `query` and a `returns`
//! type expression. The collection itself lives in [`crate::collect`]; this
//! module is config shape and load-time checks. Collectors merge by name across
//! layers and packs exactly as the rule arrays do ([`crate::config::packs`]).

use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Deserializer};

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
#[derive(Debug, Clone, PartialEq, schemars::JsonSchema)]
#[serde(untagged)]
pub enum CollectorQuery {
    /// A regex over one or more body sections.
    Regex(RegexQuery),
    /// A project Typst function over the whole task or one of its fields.
    Typst(TypstQuery),
    /// An ast-grep pattern over the working tree, through the shared engine.
    AstGrep(AstGrepQuery),
}

/// The keys that tell a collector query's kind apart, in the order tried.
const QUERY_KEYS: [&str; 3] = ["section", "typst", "ast_grep"];

/// Dispatch on the query's own key and let its struct read the table, rather
/// than derive an untagged enum: serde's untagged error erases the field, while
/// the struct keeps its `deny_unknown_fields` and missing-field messages. The
/// table is buffered first because TOML key order is not the declared order.
impl<'de> Deserialize<'de> for CollectorQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let table = toml::Table::deserialize(deserializer)?;
        if table.contains_key("section") {
            parse_query(table, Self::Regex)
        } else if table.contains_key("typst") {
            parse_query(table, Self::Typst)
        } else if table.contains_key("ast_grep") {
            parse_query(table, Self::AstGrep)
        } else if let Some(key) = table.keys().next() {
            Err(serde::de::Error::unknown_field(key, &QUERY_KEYS))
        } else {
            Err(serde::de::Error::custom(
                "a collector query needs a `section`, `typst` or `ast_grep` key",
            ))
        }
    }
}

/// One variant's struct read from the buffered table, its error naming the
/// field at fault.
fn parse_query<T, E>(
    table: toml::Table,
    variant: fn(T) -> CollectorQuery,
) -> Result<CollectorQuery, E>
where
    T: serde::de::DeserializeOwned,
    E: serde::de::Error,
{
    T::deserialize(toml::Value::Table(table))
        .map(variant)
        .map_err(|err| delegated(&err))
}

/// A delegated struct's error, carrying the field it names. Unknown, missing
/// and duplicate field messages already name it; a type mismatch says only
/// what type it found, so the buffered table's key path is kept for those.
fn delegated<E: serde::de::Error>(err: &toml::de::Error) -> E {
    let named = ["unknown field `", "missing field `", "duplicate field `"]
        .iter()
        .any(|prefix| err.message().starts_with(prefix));
    E::custom(if named {
        err.message().to_owned()
    } else {
        err.to_string()
    })
}

impl CollectorQuery {
    /// The query's kind, as `mt rules` prints it.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        if matches!(self, Self::Regex(_)) {
            "regex"
        } else if matches!(self, Self::Typst(_)) {
            "typst"
        } else {
            "ast-grep"
        }
    }
}

/// An ast-grep query: `{ ast_grep = { language, pattern, glob? } }`. The
/// shared engine runs it over the files git lists, so the collector names no
/// path set of its own.
#[derive(Debug, Clone, PartialEq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AstGrepQuery {
    /// The language, pattern and optional glob the engine compiles.
    pub ast_grep: AstGrepFields,
}

/// The body of an `ast_grep` query.
#[derive(Debug, Clone, PartialEq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AstGrepFields {
    /// The grammar to parse with: `rust` or its alias `rs` in this wave.
    pub language: String,
    /// The pattern, in ast-grep syntax.
    pub pattern: String,
    /// A glob or `@name` path set a matched file must also fall in.
    #[serde(default)]
    pub glob: Option<String>,
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
            CollectorQuery::AstGrep(query) => {
                validate_ast_grep(query, spec, &path, config, &mut problems);
            }
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

/// An ast-grep query compiles through the shared engine at load, so a bad
/// pattern, language or glob is one config error rather than one collector
/// problem per task. Its only possible result is a list of paths.
fn validate_ast_grep(
    query: &AstGrepQuery,
    spec: &CollectorSpec,
    path: &str,
    config: &Config,
    problems: &mut Vec<Problem>,
) {
    let ast = &query.ast_grep;
    let at = format!("{path}.query.ast_grep");
    let language = crate::query::check_language(&ast.language);
    if let Err(err) = &language {
        problems.push(Problem::new(format!("{at}.language: {err}")));
    }
    if language.is_ok()
        && let Err(err) = crate::query::check_pattern(&ast.language, &ast.pattern)
    {
        problems.push(Problem::new(format!("{at}.pattern: {err}")));
    }
    if let Some(glob) = &ast.glob
        && let Err(problem) = crate::query::path_filter(glob, config.compiled_dicts())
    {
        problems.push(problem.under(&format!("{at}.glob")));
    }
    if !spec.returns.is_path_list() {
        problems.push(Problem::new(format!(
            "{path}.returns: an ast-grep query returns [\"path\"], not {}",
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

#[cfg(test)]
#[path = "collect_query_tests.rs"]
mod query_tests;
