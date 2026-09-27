#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::check_commit;
use crate::commit_msg::CommitRecord;
use crate::config::parse_config;
use crate::severity::Severity;

fn record(refs: &[&str], tasks_only: bool) -> CommitRecord {
    CommitRecord {
        subject: "feat: parse".to_owned(),
        refs: refs.iter().map(|r| (*r).to_owned()).collect(),
        files: vec!["src/lib.rs".to_owned()],
        tasks_only,
    }
}

const RULE: &str = "[[commits.rules]]\nname = \"work-names-a-task\"\n\
     when = '!tasks_only'\nassert = 'has(refs)'\nmessage = \"name it\"\n";

#[test]
fn the_trailer_defaults_to_refs() {
    assert_eq!(parse_config("").unwrap().commits.trailer, "Refs");
}

#[test]
fn a_breach_carries_the_label_message_and_severity() {
    let config = parse_config(RULE).unwrap();
    let breaches = check_commit(&record(&[], false), &config);
    assert_eq!(breaches.len(), 1);
    assert_eq!(breaches[0].label, "commits.rules.work-names-a-task");
    assert_eq!(breaches[0].message, "name it");
    assert_eq!(breaches[0].severity, Severity::Warn);
}

#[test]
fn a_rule_holds_or_is_gated_out() {
    let config = parse_config(RULE).unwrap();
    assert!(check_commit(&record(&["parser"], false), &config).is_empty());
    assert!(check_commit(&record(&[], true), &config).is_empty());
}

#[test]
fn with_no_message_the_breach_names_the_assertion() {
    let config = parse_config("[[commits.rules]]\nassert = 'has(refs)'\n").unwrap();
    let breaches = check_commit(&record(&[], false), &config);
    assert_eq!(breaches[0].label, "commits.rules[0]");
    assert_eq!(breaches[0].message, "must satisfy `has(refs)`");
}

#[test]
fn validation_names_each_unusable_expression() {
    let config = parse_config(
        "[commits]\ntrailer = \" \"\n\n\
         [[commits.rules]]\nname = \"a\"\nassert = 'status == \"wip\"'\nwhen = 'has(('\n\n\
         [[commits.rules]]\nname = \"a\"\nassert = 'has(refs)'\n",
    )
    .unwrap();
    let problems = config.validate().unwrap_err();
    assert!(problems.contains("commits.trailer"), "{problems}");
    assert!(problems.contains("commits.rules[0].assert"), "{problems}");
    assert!(problems.contains("commits.rules[0].when"), "{problems}");
    assert!(
        problems.contains("commits.rules[1]: duplicate name `a`"),
        "{problems}"
    );
}

#[test]
fn a_rule_needs_an_assertion() {
    assert!(parse_config("[[commits.rules]]\nname = \"a\"\n").is_err());
}
