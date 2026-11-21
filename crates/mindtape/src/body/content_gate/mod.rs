//! Section-content gating: compile `[[tasks.body.content.rules]]` and evaluate
//! them against a task's source.
//!
//! A project declares its rules in `[tasks.body.content]` in
//! `.mindtape/config.toml` (types in [`crate::body::content`]). Compile them once
//! with [`compile_content_rules`], then call [`check_content`] once per task's
//! source text. That is the mirror image of the heading-tree gating in
//! [`crate::body::gate`], whose path compiler it shares.
//! [`artifact_gate::select_by`] picks the sections and
//! [`typst_cst::Heading::section`] spans each one.
//!
//! A `path` (the same `HeaderPathSegment` shape the heading rules use)
//! selects one or more sections. A section's *content* is everything from the
//! end of its heading down to the next heading of the same or higher level,
//! nested subsection prose and code blocks included. The assertion then runs
//! over that raw source slice: non-empty, a word-count bound, a regex the
//! slice must / must not match, or a density. A density means no
//! single unit (section, paragraph, or sentence, as `content_gate_density`'s own
//! `density_units` decides the split) may carry more than `max` matches of a
//! regex. A word is a whitespace-delimited token containing an alphanumeric
//! character. The default `count` of `all` sees markup, code and nested
//! subsection headings. `prose` drops fenced code, and `text` also masks inline
//! code. A `Scope` section that is all `#raw` is a real shape and must not read
//! as empty. Density's unit split drops code too, fenced or inline, since
//! a pattern inside a code sample is not prose.

use artifact_gate::{Matcher, select_by};
use regex::Regex;
use typst_cst::{parse_headings, tree};

use crate::body::content::{ContentAssert, ContentConfig, DensityUnit, WordCount, WordDefaults};
use crate::body::gate::{GateError, compile_path};
use crate::config::Config;
use crate::filter::Filter;
use crate::model::Task;
use crate::validate::{Location, Violation};

mod content_gate_density;
use content_gate_density::{density_violation, mask_inline_code, strip_fenced_code};

/// A compiled, ready-to-run content rule (pre-parsed filter, pre-compiled
/// regexes, pre-baked diagnostic message).
#[derive(Debug)]
pub struct CompiledContentRule {
    path: Vec<Matcher>,
    assert: CompiledAssert,
    when: Option<Filter>,
    /// Override for the default violation message.
    message: Option<String>,
    /// This rule's name if it has one, else its position. See
    /// [`crate::body::content::ContentRuleConfig::label`].
    label: String,
}

/// A compiled assertion.
#[derive(Debug)]
enum CompiledAssert {
    /// The section must contain any non-whitespace content.
    NonEmpty,
    /// The section's word count must fall inside this bound.
    Words {
        /// The fewest words the section may contain.
        min: Option<usize>,
        /// The most words the section may contain.
        max: Option<usize>,
        /// What the count counts.
        count: WordCount,
    },
    /// The section's source text must match this regex.
    MustMatch(Regex),
    /// The section's source text must not match this regex.
    MustNotMatch(Regex),
    /// No single unit of the section may carry more than `max` matches of
    /// the regex.
    Density {
        /// Compiled pattern, counted within each unit.
        re: Regex,
        /// The most matches a single unit may carry.
        max: usize,
        /// The unit the density counts over.
        per: DensityUnit,
    },
}

/// Compile content rules, pre-parsing `when` expressions and compiling regexes.
///
/// A rule that omits `count` inherits [`ContentConfig::words`]. The section
/// default reaches every rule that does not set its own.
///
/// # Errors
/// Returns [`GateError`] if any rule has an empty path, an invalid regex, or
/// an invalid `when` expression.
pub fn compile_content_rules(
    config: &ContentConfig,
) -> Result<Vec<CompiledContentRule>, GateError> {
    config
        .rules
        .iter()
        .enumerate()
        .map(|(index, rule)| compile_content_rule(rule, index, &config.words))
        .collect()
}

fn compile_content_rule(
    rule: &crate::body::content::ContentRuleConfig,
    index: usize,
    defaults: &WordDefaults,
) -> Result<CompiledContentRule, GateError> {
    if rule.path.is_empty() {
        return Err(GateError::EmptyPath);
    }
    let path = compile_path(&rule.path)?;
    let assert = match &rule.assert {
        ContentAssert::NonEmpty => CompiledAssert::NonEmpty,
        ContentAssert::Words(spec) => CompiledAssert::Words {
            min: spec.min,
            max: spec.max,
            count: spec.count.unwrap_or(defaults.count),
        },
        ContentAssert::MustMatch(p) => CompiledAssert::MustMatch(compile_regex(p)?),
        ContentAssert::MustNotMatch(p) => CompiledAssert::MustNotMatch(compile_regex(p)?),
        ContentAssert::Density(spec) => CompiledAssert::Density {
            re: compile_regex(&spec.pattern)?,
            max: spec.max,
            per: spec.per,
        },
    };
    let when = rule.when.as_deref().map(Filter::parse).transpose()?;
    Ok(CompiledContentRule {
        path,
        assert,
        when,
        message: rule.message.clone(),
        label: rule.label(index),
    })
}

impl CompiledContentRule {
    /// A breach's message and detail from the check's own words: the
    /// configured message with those words kept as the detail, or those words
    /// alone.
    fn worded(&self, default: String) -> (String, Option<String>) {
        match &self.message {
            Some(message) => (message.clone(), Some(default)),
            None => (default, None),
        }
    }
}

fn compile_regex(pattern: &str) -> Result<Regex, GateError> {
    Regex::new(pattern).map_err(|err| GateError::Regex {
        pattern: pattern.to_owned(),
        error: err.to_string(),
    })
}

/// Run compiled content rules against a task's source text.
///
/// Parses the heading structure with [`typst_cst`], selects sections by path,
/// slices each matched section's content, and applies every rule. Every
/// violation points at the matched heading's line.
#[must_use]
pub fn check_content(
    task: &Task,
    source: &str,
    rules: &[CompiledContentRule],
    config: &Config,
) -> Vec<Violation> {
    if rules.is_empty() {
        return Vec::new();
    }
    let roots = tree(parse_headings(source));
    if roots.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for rule in rules {
        if !crate::body::applies(rule.when.as_ref(), task, config) {
            continue;
        }
        for node in select_by(
            &roots,
            &rule.path,
            |n| n.heading.text.as_str(),
            |n| n.children.as_slice(),
        ) {
            let content = source.get(node.heading.section.clone()).unwrap_or_default();
            let heading = &node.heading.text;
            let Some(default) = evaluate(&rule.assert, content, heading) else {
                continue;
            };
            let (message, detail) = rule.worded(default);
            let mut violation = crate::body::body_violation(
                task,
                &rule.label,
                message,
                Location::at(task.file.clone(), node.heading.line),
                rule.when.as_ref(),
            );
            violation.detail = detail;
            out.push(violation);
        }
    }
    out
}

/// Apply a compiled assertion to a section's content, returning the default
/// violation message when it fails (and `None` when it passes).
fn evaluate(assert: &CompiledAssert, content: &str, heading: &str) -> Option<String> {
    match assert {
        CompiledAssert::NonEmpty => {
            (content.trim().is_empty()).then(|| format!("section `{heading}` must not be empty"))
        }
        CompiledAssert::Words { min, max, count } => {
            let found = word_count(content, *count);
            match (min, max) {
                (Some(min), _) if found < *min => Some(format!(
                    "section `{heading}` must contain at least {min} words (found {found})"
                )),
                (_, Some(max)) if found > *max => Some(format!(
                    "section `{heading}` must contain at most {max} words (found {found})"
                )),
                _ => None,
            }
        }
        CompiledAssert::MustMatch(re) => (!re.is_match(content))
            .then(|| format!("section `{heading}` must match `{}`", re.as_str())),
        CompiledAssert::MustNotMatch(re) => re
            .is_match(content)
            .then(|| format!("section `{heading}` must not match `{}`", re.as_str())),
        CompiledAssert::Density { re, max, per } => {
            density_violation(content, heading, re, *max, *per)
        }
    }
}

fn word_count(text: &str, count: WordCount) -> usize {
    let text = match count {
        WordCount::All => std::borrow::Cow::Borrowed(text),
        WordCount::Prose => std::borrow::Cow::Owned(strip_fenced_code(text)),
        WordCount::Text => std::borrow::Cow::Owned(mask_inline_code(&strip_fenced_code(text))),
    };
    text.split_whitespace()
        .filter(|tok| tok.chars().any(char::is_alphanumeric))
        .count()
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod words_tests;
