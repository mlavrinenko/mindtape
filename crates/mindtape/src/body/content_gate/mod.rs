//! Section-content gating: compile `[[tasks.body.content.rules]]` and evaluate
//! them against a task's source.
//!
//! Rules are declared in `[tasks.body.content]` in `.mindtape/config.toml`
//! (types in [`crate::body::content`]). Compile them once with
//! [`compile_content_rules`], then call [`check_content`] once per task's
//! source text — the mirror image of the heading-tree gating in [`crate::body::gate`],
//! which this module reuses for its path selection but not for its evaluation.
//!
//! A `path` (the same `HeaderPathSegment` shape the heading rules use)
//! selects one or more sections. A section's *content* is everything from the
//! end of its heading down to the next heading of the same or higher level —
//! nested subsection prose and code blocks included. The assertion then runs
//! over that raw source slice: non-empty, a minimum or maximum word count, a
//! regex the slice must / must not match, or a density (no single unit —
//! section, paragraph, or sentence, `content_gate_density`'s own
//! `density_units` decides the split — may carry more than `max` matches of a
//! regex). Words are
//! whitespace-delimited tokens containing an alphanumeric character, so
//! markup, code blocks, and nested subsection headings all count — a `Scope`
//! section that is all `#raw` is a real shape and must not read as empty.
//! Density is the one exception: its unit split drops fenced code entirely,
//! since a pattern inside a code sample is not prose.

use regex::Regex;
use typst_cst::{Heading, HeadingNode, parse_headings, tree};

use crate::body::content::{ContentAssert, DensityUnit};
use crate::body::gate::GateError;
use crate::config::{Config, HeaderPathSegment};
use crate::filter::Filter;
use crate::model::Task;
use crate::validate::severity::Severity;
use crate::validate::{Location, Violation};

mod content_gate_density;
use content_gate_density::density_violation;

/// A compiled, ready-to-run content rule (pre-parsed filter, pre-compiled
/// regexes, pre-baked diagnostic message).
#[derive(Debug)]
pub struct CompiledContentRule {
    path: Vec<CompiledSeg>,
    assert: CompiledAssert,
    when: Option<Filter>,
    /// Override for the default violation message.
    message: Option<String>,
    /// This rule's name if it has one, else its position — see
    /// [`crate::body::content::ContentRuleConfig::label`].
    label: String,
}

/// One compiled path segment, mirroring `artifact_gate::Matcher` with the
/// fields exposed so match evaluation can live here.
#[derive(Debug)]
enum CompiledSeg {
    /// Exact, case-sensitive, direct-child heading-text match.
    Exact(String),
    /// Regex heading-text match; `recursive = true` searches all descendants.
    Pattern {
        /// Compiled regex.
        re: Regex,
        /// Search any descendant (`true`) or only direct children (`false`).
        recursive: bool,
    },
}

/// A compiled assertion.
#[derive(Debug)]
enum CompiledAssert {
    /// The section must contain any non-whitespace content.
    NonEmpty,
    /// The section must contain at least this many words.
    MinWords(usize),
    /// The section must contain at most this many words.
    MaxWords(usize),
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
        /// Unit granularity the density is measured over.
        per: DensityUnit,
    },
}

/// Compile content rules, pre-parsing `when` expressions and compiling regexes.
///
/// # Errors
/// Returns [`GateError`] if any rule has an empty path, an invalid regex, or
/// an invalid `when` expression.
pub fn compile_content_rules(
    rules: &[crate::body::content::ContentRuleConfig],
) -> Result<Vec<CompiledContentRule>, GateError> {
    rules
        .iter()
        .enumerate()
        .map(|(index, rule)| compile_content_rule(rule, index))
        .collect()
}

fn compile_content_rule(
    rule: &crate::body::content::ContentRuleConfig,
    index: usize,
) -> Result<CompiledContentRule, GateError> {
    if rule.path.is_empty() {
        return Err(GateError::EmptyPath);
    }
    let path = rule
        .path
        .iter()
        .map(compile_segment)
        .collect::<Result<_, _>>()?;
    let assert = match &rule.assert {
        ContentAssert::NonEmpty => CompiledAssert::NonEmpty,
        ContentAssert::MinWords(n) => CompiledAssert::MinWords(*n),
        ContentAssert::MaxWords(n) => CompiledAssert::MaxWords(*n),
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

fn compile_segment(seg: &HeaderPathSegment) -> Result<CompiledSeg, GateError> {
    match seg {
        HeaderPathSegment::Exact(text) => Ok(CompiledSeg::Exact(text.clone())),
        HeaderPathSegment::Pattern { pattern, recursive } => Ok(CompiledSeg::Pattern {
            re: compile_regex(pattern)?,
            recursive: *recursive,
        }),
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
    let headings = parse_headings(source);
    if headings.is_empty() {
        return Vec::new();
    }
    let roots = tree(headings.clone());
    let mut out = Vec::new();
    for rule in rules {
        if !rule_applies(rule, task, config) {
            continue;
        }
        for node in match_path(&roots, &rule.path) {
            let content = section_content(source, &headings, node);
            let heading = &node.heading.text;
            let Some(default) = evaluate(&rule.assert, content, heading) else {
                continue;
            };
            out.push(Violation {
                task_id: task.id.clone(),
                field: rule.label.clone(),
                message: rule.message.clone().unwrap_or(default),
                location: Location::at(task.file.clone(), node.heading.line),
                severity: Severity::Error,
                kind: crate::validate::violation::ViolationKind::Other,
                reads: crate::validate::violation::reads_of(rule.when.iter()),
                unknown: None,
                dict_miss: None,
            });
        }
    }
    out
}

/// Nodes matching the full `path` from `roots` — the same traversal
/// `artifact_gate`'s path rules use (Exact is a direct-child match, a Pattern
/// with `recursive` starts from all descendants).
fn match_path<'a>(roots: &'a [HeadingNode], path: &[CompiledSeg]) -> Vec<&'a HeadingNode> {
    let Some((seg, rest)) = path.split_first() else {
        return Vec::new();
    };
    let candidates: Vec<&HeadingNode> = if is_recursive(seg) {
        all_descendants(roots)
    } else {
        roots.iter().collect()
    };
    let mut results = Vec::new();
    for entry in candidates {
        if seg_matches(seg, &entry.heading.text) {
            if rest.is_empty() {
                results.push(entry);
            } else {
                results.extend(match_path(&entry.children, rest));
            }
        }
    }
    results
}

fn all_descendants(roots: &[HeadingNode]) -> Vec<&HeadingNode> {
    let mut found = Vec::new();
    for entry in roots {
        found.push(entry);
        found.extend(all_descendants(&entry.children));
    }
    found
}

fn seg_matches(seg: &CompiledSeg, text: &str) -> bool {
    match seg {
        CompiledSeg::Exact(expected) => expected == text,
        CompiledSeg::Pattern { re, .. } => re.is_match(text),
    }
}

fn is_recursive(seg: &CompiledSeg) -> bool {
    matches!(
        seg,
        CompiledSeg::Pattern {
            recursive: true,
            ..
        }
    )
}

/// The source text of a section: everything after `node`'s heading marker up
/// to the next heading of the same or higher level (nested subsection prose
/// therefore included), or to the end of the file for the last section.
fn section_content<'a>(source: &'a str, headings: &[Heading], node: &HeadingNode) -> &'a str {
    let start = node.heading.range.end;
    // The node's own heading always appears in the flat document-order list
    // the tree was built from; if it somehow does not, treat the section as
    // everything after it. `.get` keeps the slicing total by construction.
    let end = headings
        .iter()
        .position(|h| h == &node.heading)
        .map_or(source.len(), |idx| {
            headings
                .get(idx + 1..)
                .into_iter()
                .flatten()
                .find(|h| h.depth <= node.heading.depth)
                .map_or(source.len(), |h| h.range.start)
        });
    source.get(start..end).unwrap_or_default()
}

/// Apply a compiled assertion to a section's content, returning the default
/// violation message when it fails (and `None` when it passes).
fn evaluate(assert: &CompiledAssert, content: &str, heading: &str) -> Option<String> {
    match assert {
        CompiledAssert::NonEmpty => {
            (content.trim().is_empty()).then(|| format!("section `{heading}` must not be empty"))
        }
        CompiledAssert::MinWords(n) => {
            let found = word_count(content);
            (found < *n).then(|| {
                format!("section `{heading}` must contain at least {n} words (found {found})")
            })
        }
        CompiledAssert::MaxWords(n) => {
            let found = word_count(content);
            (found > *n).then(|| {
                format!("section `{heading}` must contain at most {n} words (found {found})")
            })
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

fn word_count(text: &str) -> usize {
    text.split_whitespace()
        .filter(|tok| tok.chars().any(char::is_alphanumeric))
        .count()
}

fn rule_applies(rule: &CompiledContentRule, task: &Task, config: &Config) -> bool {
    let Some(filter) = &rule.when else {
        return true;
    };
    filter.matches(task, config).unwrap_or(false)
}

#[cfg(test)]
mod tests;
