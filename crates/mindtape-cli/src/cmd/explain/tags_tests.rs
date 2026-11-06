#![allow(clippy::unwrap_used)]

use mindtape::{Config, parse_config};

use super::{TagEntry, entries, json, render};

use crate::cmd::explain::rules::Moment;

/// Four allowed tags: `bug` read by a task rule and a commit rule, `api` read
/// by a body rule, `ship` read by a flip action, and `cli` read by none.
fn config() -> Config {
    parse_config(
        "[tasks.tags]\n\
         allowed = [\"bug\", \"api\", \"ship\", \"cli\"]\n\
         [[tasks.rules]]\n\
         name = \"bugs-need-repro\"\n\
         when = 'has_tag(\"bug\") && !terminal'\n\
         assert = 'has(repro)'\n\
         message = \"show a repro\"\n\
         [[tasks.body.content.rules]]\n\
         name = \"api-notes\"\n\
         path = [\"Changes\"]\n\
         assert = \"non-empty\"\n\
         when = 'has_tag(\"api\")'\n\
         message = \"say which endpoint\"\n\
         [[commits.rules]]\n\
         name = \"bugfix-names-bug\"\n\
         scope = \"commit\"\n\
         when = 'has_tag(\"bug\")'\n\
         assert = \"has(refs)\"\n\
         message = \"name the bug in the commit\"\n\
         [[on.flip]]\n\
         when = 'into(\"done\") && has_tag(\"ship\")'\n\
         actions = [{ run = \"echo shipped\", message = \"ship it\" }]\n",
    )
    .unwrap()
}

fn names(entries: &[TagEntry], tag: &str) -> Vec<String> {
    entries
        .iter()
        .find(|entry| entry.tag == tag)
        .map(|entry| entry.rules.iter().map(|rule| rule.name.clone()).collect())
        .unwrap_or_default()
}

#[test]
fn lists_each_allowed_tag_with_the_rules_that_read_it() {
    let entries = entries(&config(), None);

    // jscpd:ignore-start
    // The fixture's tag order and rule list are data, not a pasted block.
    let tags: Vec<&str> = entries.iter().map(|entry| entry.tag.as_str()).collect();
    assert_eq!(tags, ["bug", "api", "ship", "cli"]);
    // jscpd:ignore-end
    assert_eq!(
        names(&entries, "bug"),
        [
            "tasks.rules.bugs-need-repro",
            "commits.rules.bugfix-names-bug"
        ]
    );
    assert_eq!(
        names(&entries, "api"),
        ["tasks.body.content.rules.api-notes"]
    );
    assert_eq!(names(&entries, "ship"), ["on.flip[0].actions[0]"]);
    assert!(names(&entries, "cli").is_empty());
}

#[test]
fn a_rule_reads_tags_in_its_when_and_its_assert_only() {
    let config = parse_config(
        "[tasks.tags]\nallowed = [\"debug\", \"bug\"]\n\
         [[tasks.rules]]\nname = \"r\"\nwhen = 'has_tag(\"debug\")'\n\
         assert = 'has(title)'\nmessage = \"mentions bug\"\n",
    )
    .unwrap();

    let entries = entries(&config, None);

    assert_eq!(names(&entries, "debug"), ["tasks.rules.r"]);
    assert!(names(&entries, "bug").is_empty());
}

#[test]
fn reads_a_tag_through_has_tags_and_a_spaced_has_tag() {
    let mut toml = String::from(
        "[tasks.tags]\n\
         allowed = [\"cli\", \"ui\", \"space tag\", \"kind\", \"bug\"]\n",
    );
    for (name, when) in [
        ("cli-only", "has(tags, \"cli\")"),
        ("ui-only", "has_tag (\"ui\")"),
        ("space-tag", "has_tag(\"space tag\")"),
        ("kind-is-cli", "has(kind, \"cli\")"),
        ("mentions-bug", "contains(\"has_tag(\\\"bug\\\")\")"),
    ] {
        toml.push_str(&format!(
            "[[tasks.rules]]\nname = \"{name}\"\nwhen = '{when}'\n\
             assert = 'has(title)'\nmessage = \"{name}\"\n"
        ));
    }
    let config = parse_config(&toml).unwrap();

    let entries = entries(&config, None);

    for (tag, expected) in [
        ("cli", &["tasks.rules.cli-only"][..]),
        ("ui", &["tasks.rules.ui-only"][..]),
        ("space tag", &["tasks.rules.space-tag"][..]),
        ("kind", &[][..]),
        ("bug", &[][..]),
    ] {
        assert_eq!(names(&entries, tag), expected, "{tag}");
    }
}

#[test]
fn keeps_only_the_rules_a_moment_enforces() {
    for (moment, expected) in [
        (Moment::Add, &["tasks.rules.bugs-need-repro"][..]),
        (Moment::Commit, &["commits.rules.bugfix-names-bug"][..]),
        (Moment::Flip, &["tasks.rules.bugs-need-repro"][..]),
    ] {
        let entries = entries(&config(), Some(moment));
        assert_eq!(names(&entries, "bug"), expected, "{moment:?}");
    }
}

#[test]
fn keeps_only_the_tags_a_moment_reads() {
    for (moment, expected) in [
        (None, &["bug", "api", "ship", "cli"][..]),
        (Some(Moment::Add), &["bug", "api"][..]),
        (Some(Moment::Commit), &["bug"][..]),
        (Some(Moment::Flip), &["bug", "api", "ship"][..]),
    ] {
        let entries = entries(&config(), moment);
        let tags: Vec<&str> = entries.iter().map(|entry| entry.tag.as_str()).collect();
        assert_eq!(tags, expected, "{moment:?}");
    }
}

#[test]
fn renders_a_message_under_its_rule_and_an_area_label_for_an_unread_tag() {
    let entries = entries(&config(), None);

    assert_eq!(
        render(&entries, None),
        "\
bug   tasks.rules.bugs-need-repro
      show a repro
bug   commits.rules.bugfix-names-bug
      name the bug in the commit
api   tasks.body.content.rules.api-notes
      say which endpoint
ship  on.flip[0].actions[0]
      ship it
cli   area label
"
    );
}

#[test]
fn says_so_when_the_list_is_empty() {
    assert_eq!(render(&[], None), "no tag list; every tag is allowed\n");
}

#[test]
fn says_so_when_no_tag_is_read_at_the_moment() {
    let config = parse_config("[tasks.tags]\nallowed = [\"cli\"]\n").unwrap();
    let entries = entries(&config, Some(Moment::Add));

    assert!(entries.is_empty());
    assert_eq!(
        render(&entries, Some(Moment::Add)),
        "no tags are read at this moment\n"
    );
    assert_eq!(
        json(&entries),
        serde_json::json!({
            "schema_version": mindtape::SCHEMA_VERSION,
            "tags": {},
        })
    );
}

#[test]
fn json_carries_the_same_map_under_schema_version() {
    let document = json(&entries(&config(), None));

    assert_eq!(
        document,
        serde_json::json!({
            "schema_version": mindtape::SCHEMA_VERSION,
            "tags": {
                "bug": [
                    {
                        "rule": "tasks.rules.bugs-need-repro",
                        "message": "show a repro",
                    },
                    {
                        "rule": "commits.rules.bugfix-names-bug",
                        "message": "name the bug in the commit",
                    },
                ],
                "api": [
                    {
                        "rule": "tasks.body.content.rules.api-notes",
                        "message": "say which endpoint",
                    },
                ],
                "ship": [
                    {
                        "rule": "on.flip[0].actions[0]",
                        "message": "ship it",
                    },
                ],
                "cli": [],
            },
        })
    );
}

#[test]
fn json_under_a_moment_holds_only_the_tags_that_step_reads() {
    let document = json(&entries(&config(), Some(Moment::Commit)));

    assert_eq!(
        document,
        serde_json::json!({
            "schema_version": mindtape::SCHEMA_VERSION,
            "tags": {
                "bug": [
                    {
                        "rule": "commits.rules.bugfix-names-bug",
                        "message": "name the bug in the commit",
                    },
                ],
            },
        })
    );
}
