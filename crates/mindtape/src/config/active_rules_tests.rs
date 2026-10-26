#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{ActiveRule, RuleSection, active_rules};
use crate::config::parse_config;
use crate::rules::CheckSpec;
use crate::validate::severity::Severity;

fn rules(toml: &str) -> Vec<ActiveRule> {
    active_rules(&parse_config(toml).unwrap())
}

#[test]
fn a_check_rule_lists_its_function_and_no_message() {
    let listed = rules(
        "[[tasks.rules]]\nname = \"goals\"\nmessage = \"ignored\"\nseverity = \"error\"\n\
         check = { typst = \"lib.typ\", function = \"check-goals\", field = \"x-goals\" }\n",
    );

    assert_eq!(
        listed,
        [ActiveRule {
            section: RuleSection::TasksRules,
            name: "tasks.rules.goals".to_owned(),
            severity: Severity::Error,
            message: None,
            when: None,
            assertion: None,
            check: Some(CheckSpec {
                typst: "lib.typ".to_owned(),
                function: "check-goals".to_owned(),
                field: Some("x-goals".to_owned()),
                root: None,
            }),
        }]
    );
}

#[test]
fn an_unnamed_commit_rule_warns_with_the_engine_advice() {
    let listed = rules("[[commits.rules]]\nassert = 'has(refs)'\nwhen = '!tasks_only'\n");

    assert_eq!(
        listed,
        [ActiveRule {
            section: RuleSection::CommitsRules,
            name: "commits.rules[0]".to_owned(),
            severity: Severity::Warn,
            message: Some("must satisfy `has(refs)`".to_owned()),
            when: Some("!tasks_only".to_owned()),
            assertion: Some("has(refs)".to_owned()),
            check: None,
        }]
    );
}

#[test]
fn body_rules_report_as_errors_with_their_own_message_or_none() {
    let listed = rules(
        "[[tasks.body.headers.rules]]\nname = \"one-problem\"\naction = \"unique\"\n\
         path = [\"Problem\"]\nwhen = '!terminal'\nmessage = \"keep one Problem\"\n\
         [[tasks.body.content.rules]]\nname = \"filled\"\npath = [\"Problem\"]\n\
         assert = \"non-empty\"\n\
         [[tasks.body.content.rules]]\npath = [\"TO-BE\"]\nassert = \"non-empty\"\n",
    );

    let summary: Vec<_> = listed
        .iter()
        .map(|rule| {
            (
                rule.section,
                rule.name.as_str(),
                rule.severity,
                rule.message.as_deref(),
                rule.when.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            (
                RuleSection::BodyHeaders,
                "tasks.body.headers.rules.one-problem",
                Severity::Error,
                Some("keep one Problem"),
                Some("!terminal"),
            ),
            (
                RuleSection::BodyContent,
                "tasks.body.content.rules.filled",
                Severity::Error,
                None,
                None,
            ),
            (
                RuleSection::BodyContent,
                "tasks.body.content.rules[1]",
                Severity::Error,
                None,
                None,
            ),
        ]
    );
    assert!(listed.iter().all(|rule| rule.assertion.is_none()));
}

/// A gate refuses the flip and a plain `run` only warns. An action with no
/// message is not listed, and an empty `when` lists as none.
#[test]
fn flip_actions_with_a_message_list_by_their_path() {
    let listed = rules(
        "[[on.flip]]\nwhen = ''\nactions = [{ show = \"hi\" }]\n\
         [[on.flip]]\nwhen = 'into(\"done\")'\nactions = [\
         { run = \"true\" }, { run = \"make\", message = \"fix the build\" }, \
         { gate = true, message = \"not yet\" }]\n\
         [[on.flip]]\nwhen = ''\nactions = [{ run = \"lint\", gate = true, message = \"lint\" }]\n",
    );

    let summary: Vec<_> = listed
        .iter()
        .map(|rule| {
            (
                rule.name.as_str(),
                rule.severity,
                rule.message.as_deref(),
                rule.when.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            (
                "on.flip[1].actions[1]",
                Severity::Warn,
                Some("fix the build"),
                Some("into(\"done\")"),
            ),
            (
                "on.flip[1].actions[2]",
                Severity::Error,
                Some("not yet"),
                Some("into(\"done\")"),
            ),
            ("on.flip[2].actions[0]", Severity::Error, Some("lint"), None),
        ]
    );
    assert!(
        listed
            .iter()
            .all(|rule| rule.section == RuleSection::OnFlip && rule.check.is_none())
    );
}

#[test]
fn sections_list_in_config_show_origin_order() {
    let listed = rules(
        "[[on.flip]]\nwhen = ''\nactions = [{ gate = true, message = \"no\" }]\n\
         [[commits.rules]]\nassert = 'has(refs)'\n\
         [[tasks.body.content.rules]]\npath = [\"A\"]\nassert = \"non-empty\"\n\
         [[tasks.body.headers.rules]]\naction = \"require\"\npath = [\"A\"]\n\
         [[tasks.rules]]\nassert = 'has(title)'\n",
    );

    let sections: Vec<_> = listed.iter().map(|rule| rule.section).collect();
    assert_eq!(
        sections,
        [
            RuleSection::TasksRules,
            RuleSection::BodyHeaders,
            RuleSection::BodyContent,
            RuleSection::CommitsRules,
            RuleSection::OnFlip,
        ]
    );
}

#[test]
fn a_section_serializes_as_its_dotted_path() {
    let paths: Vec<String> = [
        RuleSection::TasksRules,
        RuleSection::BodyHeaders,
        RuleSection::BodyContent,
        RuleSection::CommitsRules,
        RuleSection::OnFlip,
    ]
    .iter()
    .map(|section| serde_json::to_string(section).unwrap())
    .collect();

    assert_eq!(
        paths,
        [
            "\"tasks.rules\"",
            "\"tasks.body.headers.rules\"",
            "\"tasks.body.content.rules\"",
            "\"commits.rules\"",
            "\"on.flip\"",
        ]
    );
}

#[test]
fn a_severity_labels_and_serializes_as_its_config_word() {
    for (severity, word) in [(Severity::Warn, "warn"), (Severity::Error, "error")] {
        assert_eq!(severity.label(), word);
        assert_eq!(
            serde_json::to_string(&severity).unwrap(),
            format!("\"{word}\"")
        );
    }
}
