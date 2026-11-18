#![allow(clippy::unwrap_used)]

use super::{matching_rm_hooks, validate_rm_hooks};
use crate::config::{Config, parse_config};
use crate::graph::ready::Readiness;
use crate::model::{Status, Task};
use crate::test_support::bare_task;

/// A task at `status` carrying `tags`.
fn task(status: &str, tags: &[&str]) -> Task {
    let mut task = bare_task("t");
    task.status = Some(Status {
        name: status.to_owned(),
        date: None,
        note: None,
    });
    task.tags = tags.iter().map(|tag| (*tag).to_owned()).collect();
    task
}

fn config(toml: &str) -> Config {
    parse_config(toml).unwrap()
}

/// The `when` of each hook that fires for `task`, in declaration order.
fn fired(config: &Config, task: &Task) -> Vec<String> {
    matching_rm_hooks(config, task, &Readiness::empty())
        .unwrap()
        .iter()
        .map(|hook| hook.when.clone())
        .collect()
}

/// Every problem `validate_rm_hooks` reports, joined.
fn problems(toml: &str) -> String {
    validate_rm_hooks(&config(toml))
        .iter()
        .map(|problem| problem.text.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}

#[test]
fn a_hook_fires_on_the_tasks_its_when_matches() {
    let config = config(
        "[[on.rm]]\nwhen = 'has_tag(\"chore\")'\nactions = [{ show = \"a\" }]\n\
         [[on.rm]]\nwhen = 'status == \"done\"'\nactions = [{ show = \"b\" }]\n\
         [[on.rm]]\nwhen = ''\nactions = [{ show = \"c\" }]\n",
    );
    assert_eq!(
        fired(&config, &task("done", &["chore"])),
        [r#"has_tag("chore")"#, r#"status == "done""#, ""]
    );
    assert_eq!(fired(&config, &task("wip", &[])), [""]);
    assert!(fired(&Config::default(), &task("wip", &[])).is_empty());
}

#[test]
fn an_unevaluable_when_names_its_section_and_index() {
    let config = config("[[on.rm]]\nwhen = 'no_such_field == 1'\nactions = [{ show = \"x\" }]\n");
    let err = matching_rm_hooks(&config, &task("wip", &[]), &Readiness::empty()).unwrap_err();
    assert_eq!((err.section, err.index), ("on.rm", 0));
    assert!(err.to_string().contains("on.rm[0].when"), "{err}");
}

/// Every problem `validate_rm_hooks` reports, each from a config that is
/// otherwise well-formed.
#[test]
fn validation_reports_each_problem() {
    let bad_when = "[[on.rm]]\nwhen = 'no_such_field == 1'\nactions = [{ show = \"x\" }]\n";
    assert!(problems(bad_when).contains("on.rm[0].when"));
    assert!(problems("[[on.rm]]\nwhen = ''\nactions = []\n").contains("no actions"));
    let ghost = "[[on.rm]]\nwhen = ''\nactions = [{ list = \"@ghost\" }]\n";
    assert!(problems(ghost).contains("on.rm[0].actions[0].list"));
    let no_kind = "[[on.rm]]\nwhen = ''\nactions = [{ title = \"x\" }]\n";
    assert!(problems(no_kind).contains("on.rm[0].actions[0]"));
    let blank = "[[on.rm]]\nwhen = ''\nactions = [{ run = \"\" }]\n";
    assert!(problems(blank).contains("on.rm[0].actions[0].run: empty command"));
}

/// A removal stamps no status, so `into(...)` is not a filter function here:
/// the load refuses it rather than letting the hook quietly never fire.
#[test]
fn into_is_not_a_removal_predicate() {
    let toml = "[[on.rm]]\nwhen = 'into(\"done\")'\nactions = [{ show = \"x\" }]\n";
    assert!(problems(toml).contains("on.rm[0].when"));
    assert!(config(toml).validate().is_err());
}

/// A valid hook passes both the module check and the whole-config check
/// `mt check` runs, whether its gate runs a command or refuses on `when`
/// alone.
#[test]
fn a_valid_hook_passes_validation() {
    for actions in [
        r#"{ list = "@wip", title = "T" }, { run = "true", gate = true }"#,
        r#"{ gate = true, message = "clean the worktree first" }"#,
    ] {
        let toml = format!(
            "[views.wip]\nfilter = 'status == \"wip\"'\n\
             [[on.rm]]\nname = \"clean\"\nwhen = 'has_tag(\"chore\") && !terminal'\n\
             actions = [{actions}]\n"
        );
        assert!(validate_rm_hooks(&config(&toml)).is_empty(), "{toml}");
        assert!(config(&toml).validate().is_ok(), "{toml}");
    }
}

/// Two entries sharing a name can only come from one file, and the whole-config
/// check reports both.
#[test]
fn two_hooks_in_one_file_cannot_share_a_name() {
    let config = config(
        "[[on.rm]]\nname = \"clean\"\nwhen = ''\nactions = [{ show = \"a\" }]\n\
         [[on.rm]]\nname = \"clean\"\nwhen = ''\nactions = [{ show = \"b\" }]\n",
    );
    let err = config.validate().unwrap_err();
    assert!(err.text.contains("on.rm"), "{err}");
}

/// A malformed hook is a load error, so no command ever reaches the run path.
#[test]
fn a_malformed_hook_fails_config_validation() {
    let toml = "[[on.rm]]\nwhen = 'no_such_field == 1'\nactions = [{ show = \"x\" }]\n";
    assert!(config(toml).validate().is_err());
}
