//! The shared per-source entry point for body-rule gating.
//!
//! [`crate::gate`] checks heading STRUCTURE and [`crate::content_gate`] checks
//! section CONTENT; both take one task's source and a pre-compiled rule set.
//! [`check_body_rules`] runs the two in the order `mt check` always has, so
//! `mt check` and the candidate seam ([`crate::candidate`]) call one function
//! rather than each carrying its own copy of "run headers, then content." A
//! rule added to either gate is picked up by both callers for free.

use crate::config::Config;
use crate::content_gate::{CompiledContentRule, check_content};
use crate::gate::{CompiledRule, check_headers};
use crate::model::Task;
use crate::validate::Violation;

/// One source's compiled body rules — `[[tasks.body.headers.rules]]` and
/// `[[tasks.body.content.rules]]` — compiled once per run (`mt check`) or once
/// per write (the candidate seam) and handed to every [`check_body_rules`]
/// call that follows.
#[derive(Debug, Clone, Copy, Default)]
pub struct BodyRules<'a> {
    /// Compiled `[[tasks.body.headers.rules]]`.
    pub headers: &'a [CompiledRule],
    /// Compiled `[[tasks.body.content.rules]]`.
    pub content: &'a [CompiledContentRule],
}

/// Run the header-structure gate then the section-content gate over `task`'s
/// `source`, in that order, and return every violation either one reports.
#[must_use]
pub fn check_body_rules(
    task: &Task,
    source: &str,
    rules: BodyRules<'_>,
    config: &Config,
) -> Vec<Violation> {
    let mut violations = check_headers(task, source, rules.headers, config);
    violations.extend(check_content(task, source, rules.content, config));
    violations
}

#[cfg(test)]
#[path = "body_gate_tests.rs"]
mod tests;
