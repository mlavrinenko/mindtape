#![allow(clippy::unwrap_used)]

use mindtape::{RuleSection, active_rules, parse_config};

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
        render(&active_rules(&config)),
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
        render(&active_rules(&config)),
        "warn   tasks.rules.ñ   a\nwarn   tasks.rules.ab  b\n"
    );
}

#[test]
fn says_so_when_the_list_is_empty() {
    assert_eq!(render(&[]), "no rules apply\n");
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
