use super::Diag;
use crate::diag::plain;

#[test]
fn a_count_line_pluralises_its_nouns() {
    assert_eq!(plain(&Diag::problems(1, 1)), "error: 1 problem in 1 task\n");
    assert_eq!(
        plain(&Diag::problems(3, 2)),
        "error: 3 problems in 2 tasks\n"
    );
    assert_eq!(
        plain(&Diag::commit_message_problems(1)),
        "error: 1 commit message problem\n"
    );
    assert_eq!(
        plain(&Diag::commits_broke_rules(2)),
        "error: 2 commits broke commit rules\n"
    );
}

#[test]
fn a_command_that_ran_ends_in_its_exit_status() {
    let failed = Diag::refused("gate failed").value("scripts/test");
    assert_eq!(
        plain(&failed.clone().exit(Some(1))),
        "error: gate failed: 'scripts/test': exit 1\n"
    );
    assert_eq!(
        plain(&failed.exit(None)),
        "error: gate failed: 'scripts/test': terminated by signal\n"
    );
}
