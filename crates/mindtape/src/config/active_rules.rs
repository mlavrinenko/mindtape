//! [`active_rules`]: every rule a loaded config applies, one [`ActiveRule`]
//! each, for `mt rules`.
//!
//! It reads the typed [`Config`], whose four rule arrays already hold what
//! [`crate::config::packs`] left after packs, same-name overrides and
//! `disable`, so the listing needs no merge of its own. The `[[on.flip]]`
//! actions that carry a `message` join them, since a flip gate refuses with
//! that message the way a rule does.

use serde::{Serialize, Serializer};

use crate::body::BODY_RULE_SEVERITY;
use crate::body::headers::rule_label;
use crate::config::Config;
use crate::config::commits::CommitRuleSpec;
use crate::config::hooks::FlipAction;
use crate::rules::{CheckSpec, RuleSpec};
use crate::validate::severity::Severity;

/// The config array a rule comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleSection {
    /// `[[tasks.rules]]`.
    TasksRules,
    /// `[[tasks.body.headers.rules]]`.
    BodyHeaders,
    /// `[[tasks.body.content.rules]]`.
    BodyContent,
    /// `[[commits.rules]]`.
    CommitsRules,
    /// `[[on.flip]]`, one entry per action.
    OnFlip,
}

impl RuleSection {
    /// The section's dotted TOML path, which leads each of its rule names.
    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::TasksRules => "tasks.rules",
            Self::BodyHeaders => "tasks.body.headers.rules",
            Self::BodyContent => "tasks.body.content.rules",
            Self::CommitsRules => "commits.rules",
            Self::OnFlip => "on.flip",
        }
    }
}

impl Serialize for RuleSection {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.path())
    }
}

/// One rule as the engine applies it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActiveRule {
    /// The array the rule comes from.
    pub section: RuleSection,
    /// The dotted name `mt config show --origin` prints, such as
    /// `tasks.rules.imperative-title` or `tasks.rules[3]` for an unnamed
    /// rule. A flip action has no name, so it reads `on.flip[0].actions[1]`,
    /// its path in a config error.
    pub name: String,
    /// The severity a breach reports at: the rule's own, else the level the
    /// engine applies to its section.
    pub severity: Severity,
    /// What a breach prints, when that text is fixed: the rule's `message`,
    /// else for an `assert` rule the engine's `must satisfy` advice. `None`
    /// when each breach words its own: a `check` rule prints what its
    /// function returns, and a body rule with no `message` prints its gate's
    /// text for the heading at fault.
    pub message: Option<String>,
    /// The filter gating which tasks, commits or flips the rule applies to.
    pub when: Option<String>,
    /// The `assert` expression of a `[[tasks.rules]]` or `[[commits.rules]]`
    /// entry.
    #[serde(rename = "assert")]
    pub assertion: Option<String>,
    /// The Typst function a `check` rule calls.
    pub check: Option<CheckSpec>,
}

/// Every rule `config` applies, in the order `mt config show --origin` lists
/// them: `[[tasks.rules]]`, the header and content body rules and
/// `[[commits.rules]]`, then each `[[on.flip]]` action that carries a
/// `message`.
#[must_use]
pub fn active_rules(config: &Config) -> Vec<ActiveRule> {
    let mut out: Vec<ActiveRule> = config
        .tasks
        .rules
        .iter()
        .enumerate()
        .map(|(index, rule)| task_rule(index, rule))
        .collect();
    let body = &config.tasks.body;
    let headers = body.headers.rules.iter();
    let content = body.content.rules.iter();
    let arrays: [(RuleSection, Vec<BodyParts<'_>>); 2] = [
        (
            RuleSection::BodyHeaders,
            headers
                .map(|r| (r.name.as_deref(), &r.message, &r.when))
                .collect(),
        ),
        (
            RuleSection::BodyContent,
            content
                .map(|r| (r.name.as_deref(), &r.message, &r.when))
                .collect(),
        ),
    ];
    for (section, rules) in arrays {
        out.extend(
            rules
                .into_iter()
                .enumerate()
                .map(|(index, parts)| body_rule(section, index, parts)),
        );
    }
    out.extend(
        config
            .commits
            .rules
            .iter()
            .enumerate()
            .map(|(index, rule)| commit_rule(index, rule)),
    );
    out.extend(flip_actions(config));
    out
}

/// One `[[commits.rules]]` entry.
fn commit_rule(index: usize, rule: &CommitRuleSpec) -> ActiveRule {
    ActiveRule {
        section: RuleSection::CommitsRules,
        name: rule_label(
            rule.name.as_deref(),
            RuleSection::CommitsRules.path(),
            index,
        ),
        severity: rule.severity,
        message: Some(rule.advice()),
        when: rule.when.clone(),
        assertion: Some(rule.assertion.clone()),
        check: None,
    }
}

/// One `[[tasks.rules]]` entry. A `check` rule prints what its function
/// returns and ignores `message`, so it lists none.
fn task_rule(index: usize, rule: &RuleSpec) -> ActiveRule {
    ActiveRule {
        section: RuleSection::TasksRules,
        name: rule_label(rule.name.as_deref(), RuleSection::TasksRules.path(), index),
        severity: rule.severity,
        message: rule.check.is_none().then(|| rule.advice()),
        when: rule.when.clone(),
        assertion: rule.assertion.clone(),
        check: rule.check.clone(),
    }
}

/// A body rule's `name`, `message` and `when`.
type BodyParts<'a> = (Option<&'a str>, &'a Option<String>, &'a Option<String>);

/// One header or content body rule, at the severity both gates report.
fn body_rule(section: RuleSection, index: usize, parts: BodyParts<'_>) -> ActiveRule {
    let (name, message, when) = parts;
    ActiveRule {
        section,
        name: rule_label(name, section.path(), index),
        severity: BODY_RULE_SEVERITY,
        message: message.clone(),
        when: when.clone(),
        assertion: None,
        check: None,
    }
}

/// Each `[[on.flip]]` action that carries a `message`. `mt flip` refuses on
/// a gate and only warns on a plain `run` (`cmd/flip/hooks.rs` in the CLI),
/// so a gate is an error and a `run` a warning.
fn flip_actions(config: &Config) -> Vec<ActiveRule> {
    let mut out = Vec::new();
    for (index, hook) in config.on.flip.iter().enumerate() {
        let when = Some(hook.when.clone()).filter(|when| !when.is_empty());
        for (slot, action) in hook.actions.iter().enumerate() {
            let Some(message) = &action.message else {
                continue;
            };
            out.push(ActiveRule {
                section: RuleSection::OnFlip,
                name: format!("on.flip[{index}].actions[{slot}]"),
                severity: flip_severity(action),
                message: Some(message.clone()),
                when: when.clone(),
                assertion: None,
                check: None,
            });
        }
    }
    out
}

const fn flip_severity(action: &FlipAction) -> Severity {
    if action.gate {
        Severity::Error
    } else {
        Severity::Warn
    }
}

#[cfg(test)]
#[path = "active_rules_tests.rs"]
mod tests;
