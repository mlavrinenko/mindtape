#![allow(clippy::unwrap_used)]

use mindtape::{DerivedField, RuleSection, active_rules, parse_config};

use super::{Moment, render};

#[test]
fn a_rule_with_no_fixed_message_says_who_words_its_breaches() {
    let config = parse_config(
        "[[tasks.rules]]\nname = \"goals\"\nseverity = \"error\"\n\
         check = { typst = \"truth/lib.typ\", function = \"check-goals\" }\n\
         [[tasks.body.content.rules]]\npath = [\"Problem\"]\nassert = \"non-empty\"\n",
    )
    .unwrap();

    assert_eq!(
        render(&active_rules(&config), &[]),
        "\
error  tasks.rules.goals            (messages from check-goals in truth/lib.typ)
error  tasks.body.content.rules[0]  (no message set)
"
    );
}

/// The name column pads by characters, so a wide name still lines up.
#[test]
fn pads_the_name_column_by_characters() {
    let config = parse_config(
        "[[tasks.rules]]\nname = \"ñ\"\nassert = 'has(title)'\nmessage = \"a\"\n\
         [[tasks.rules]]\nname = \"ab\"\nassert = 'has(title)'\nmessage = \"b\"\n",
    )
    .unwrap();

    assert_eq!(
        render(&active_rules(&config), &[]),
        "warn   tasks.rules.ñ   a\nwarn   tasks.rules.ab  b\n"
    );
}

#[test]
fn says_so_when_the_list_is_empty() {
    assert_eq!(render(&[], &[]), "no rules apply\n");
}

#[test]
fn each_moment_enforces_the_sections_its_command_checks() {
    let sections = [
        RuleSection::TasksRules,
        RuleSection::BodyHeaders,
        RuleSection::BodyContent,
        RuleSection::CommitsRules,
        RuleSection::OnFlip,
    ];
    let table = [
        (Moment::Add, [true, true, true, false, false]),
        (Moment::Commit, [false, false, false, true, false]),
        (Moment::Flip, [true, true, true, false, true]),
    ];
    for (moment, expected) in table {
        let got = sections.map(|section| moment.enforces(section));
        assert_eq!(got, expected, "{moment:?}");
    }
}

#[test]
fn a_collector_line_names_its_kind_detail_and_readers() {
    let cases = [
        (
            DerivedField {
                name: "derived.title".to_owned(),
                kind: "typst".to_owned(),
                detail: Some("lib.typ::title".to_owned()),
                returns: "string".to_owned(),
                origin: "config".to_owned(),
                readers: Vec::new(),
            },
            ["typst lib.typ::title", "derived.title"],
        ),
        (
            DerivedField {
                name: "derived.callers".to_owned(),
                kind: "ast-grep".to_owned(),
                detail: None,
                returns: "[path]".to_owned(),
                origin: "config".to_owned(),
                readers: vec!["tasks.rules.small-fanout".to_owned()],
            },
            ["ast-grep", "read by tasks.rules.small-fanout"],
        ),
    ];
    for (derived, expected) in cases {
        let text = render(&[], std::slice::from_ref(&derived));
        for part in expected {
            assert!(text.contains(part), "{part}: {text}");
        }
    }
}
