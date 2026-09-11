//! Section-content gating: compile `[[tasks.body.content.rules]]` and evaluate
//! them against a task's source.
//!
//! Rules are declared in `[tasks.body.content]` in `.mindtape/config.toml`
//! (types in [`crate::body_content`]). Compile them once with
//! [`compile_content_rules`], then call [`check_content`] once per task's
//! source text — the mirror image of the heading-tree gating in [`crate::gate`],
//! which this module reuses for its path selection but not for its evaluation.
//!
//! A `path` (the same `HeaderPathSegment` shape the heading rules use)
//! selects one or more sections. A section's *content* is everything from the
//! end of its heading down to the next heading of the same or higher level —
//! nested subsection prose and code blocks included. The assertion then runs
//! over that raw source slice: non-empty, a minimum word count, or a regex
//! the slice must / must not match. Words are whitespace-delimited tokens
//! containing an alphanumeric character, so markup, code blocks, and nested
//! subsection headings all count — a `Scope` section that is all `#raw` is a
//! real shape and must not read as empty.

use regex::Regex;
use typst_cst::{Heading, HeadingNode, parse_headings, tree};

use crate::body_content::ContentAssert;
use crate::config::{Config, HeaderPathSegment};
use crate::filter::Filter;
use crate::gate::GateError;
use crate::model::Task;
use crate::validate::{Location, Violation};

/// A compiled, ready-to-run content rule (pre-parsed filter, pre-compiled
/// regexes, pre-baked diagnostic message).
#[derive(Debug)]
pub struct CompiledContentRule {
    path: Vec<CompiledSeg>,
    assert: CompiledAssert,
    when: Option<Filter>,
    /// Override for the default violation message.
    message: Option<String>,
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
    /// The section's source text must match this regex.
    MustMatch(Regex),
    /// The section's source text must not match this regex.
    MustNotMatch(Regex),
}

/// Compile content rules, pre-parsing `when` expressions and compiling regexes.
///
/// # Errors
/// Returns [`GateError`] if any rule has an empty path, an invalid regex, or
/// an invalid `when` expression.
pub fn compile_content_rules(
    rules: &[crate::body_content::ContentRuleConfig],
) -> Result<Vec<CompiledContentRule>, GateError> {
    rules.iter().map(compile_content_rule).collect()
}

fn compile_content_rule(
    rule: &crate::body_content::ContentRuleConfig,
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
        ContentAssert::MustMatch(p) => CompiledAssert::MustMatch(compile_regex(p)?),
        ContentAssert::MustNotMatch(p) => CompiledAssert::MustNotMatch(compile_regex(p)?),
    };
    let when = rule.when.as_deref().map(Filter::parse).transpose()?;
    Ok(CompiledContentRule {
        path,
        assert,
        when,
        message: rule.message.clone(),
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
                field: "body.content".to_owned(),
                message: rule.message.clone().unwrap_or(default),
                location: Some(Location {
                    file: task.file.clone(),
                    line: node.heading.line,
                }),
                severity: None,
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
        CompiledAssert::MustMatch(re) => (!re.is_match(content))
            .then(|| format!("section `{heading}` must match `{}`", re.as_str())),
        CompiledAssert::MustNotMatch(re) => re
            .is_match(content)
            .then(|| format!("section `{heading}` must not match `{}`", re.as_str())),
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
#[path = "content_gate_tests.rs"]
mod tests;
