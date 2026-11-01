#![allow(clippy::unwrap_used)]

use mindtape::{Config, parse_config};

use super::{TagEntry, entries, json, render};

use crate::cmd::rules::Moment;

/// Three allowed tags: `bug` read by a task rule and a commit rule, `api` read
/// by a body rule, and `cli` read by none.
fn config() -> Config {
    parse_config(
        "[tasks.tags]\n\
         allowed = [\"bug\", \"api\", \"cli\"]\n\
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
         message = \"name the bug in the commit\"\n",
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
    assert_eq!(tags, ["bug", "api", "cli"]);
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
fn renders_a_message_under_its_rule_and_an_area_label_for_an_unread_tag() {
    let entries = entries(&config(), None);

    assert_eq!(
        render(&entries),
        "\
bug  tasks.rules.bugs-need-repro
     show a repro
bug  commits.rules.bugfix-names-bug
     name the bug in the commit
api  tasks.body.content.rules.api-notes
     say which endpoint
cli  area label
"
    );
}

#[test]
fn says_so_when_the_list_is_empty() {
    assert_eq!(render(&[]), "no tag list; every tag is allowed\n");
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
                "cli": [],
            },
        })
    );
}
