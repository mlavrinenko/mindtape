//! Load-time checks for `[[on.flip]]` and `[[on.rm]]`.
//!
//! A malformed hook is one config error at load, not a surprise halfway
//! through a batch write with some tasks already changed. The check desugars a
//! `when` that has an `into(...)`, parses it, and dry-runs it against a probe
//! task. Each action must resolve to exactly one kind, and a `list` filter must
//! parse. The check compares a flip's `into(...)` names with the status
//! legend.
//!
//! Both hook arrays share [`validate_hooks`], so a rule added here covers
//! `[[on.rm]]` as well.

use crate::config::Config;
use crate::model::Task;
use crate::problem::Problem;
use crate::suggest::Unknown;

use super::{ActionKind, FlipAction, HookFields, desugar_into, into_targets};

/// Check every `[[on.flip]]` entry at config load, returning one message per
/// problem, prefixed with its dotted TOML path.
pub(crate) fn validate_flip_hooks(config: &Config) -> Vec<Problem> {
    let mut problems = into_problems(config);
    problems.extend(validate_hooks(&config.on.flip, "on.flip", config, |when| {
        desugar_into(when, "")
    }));
    problems
}

/// Check every `[[on.rm]]` entry at config load, returning one message per
/// problem, prefixed with its dotted TOML path.
pub(crate) fn validate_rm_hooks(config: &Config) -> Vec<Problem> {
    validate_hooks(&config.on.rm, "on.rm", config, str::to_owned)
}

/// Every `into(...)` name a flip hook names that the status legend does not
/// declare. An empty legend unconstrains the names rather than rejecting every
/// one.
fn into_problems(config: &Config) -> Vec<Problem> {
    let legend = &config.tasks.status;
    let mut problems = Vec::new();
    for (index, hook) in config.on.flip.iter().enumerate() {
        for name in into_targets(&hook.when) {
            if !legend.is_empty() && !legend.contains(&name) {
                let known = legend.values();
                problems.push(
                    Problem::new(format!(
                        "on.flip[{index}].when: unknown status: {} in into()",
                        crate::config::quote(&name)
                    ))
                    .with(Unknown::near(&name, &known)),
                );
            }
        }
    }
    problems
}

/// Check one hook array: each `when` parses and dry-runs, each entry declares
/// an action, and each action resolves to one kind with a `list` filter that
/// parses. `prepare` rewrites a `when` before it parses, which is where a
/// flip's `into(...)` becomes a filter constant.
fn validate_hooks<H: HookFields>(
    hooks: &[H],
    section: &str,
    config: &Config,
    prepare: impl Fn(&str) -> String,
) -> Vec<Problem> {
    let probe = Task::default();
    let mut problems = Vec::new();
    for (index, hook) in hooks.iter().enumerate() {
        let (_, when, actions) = hook.parts();
        if let Some(problem) = crate::filter::expr_problem_in_set(&prepare(when), &probe, config) {
            problems.push(problem.under(&format!("{section}[{index}].when")));
        }
        if actions.is_empty() {
            problems.push(Problem::new(format!(
                "{section}[{index}]: no actions declared"
            )));
        }
        problems.extend(action_problems(section, index, actions, &probe, config));
    }
    problems
}

/// Load-time problems in one hook's actions: an unresolvable kind, or a `list`
/// filter that does not parse or evaluate. `section` is the hook array the
/// entry sits in, `on.flip` or `on.rm`.
fn action_problems(
    section: &str,
    index: usize,
    actions: &[FlipAction],
    probe: &Task,
    config: &Config,
) -> Vec<Problem> {
    let mut problems = Vec::new();
    for (slot, action) in actions.iter().enumerate() {
        let at = format!("{section}[{index}].actions[{slot}]");
        match action.kind() {
            Err(message) => problems.push(Problem::new(format!("{at}: {message}"))),
            Ok(ActionKind::List { filter, .. }) => {
                if let Some(problem) = crate::filter::expr_problem_in_set(filter, probe, config) {
                    problems.push(problem.under(&format!("{at}.list")));
                }
            }
            Ok(ActionKind::Run {
                command: Some(command),
                ..
            }) if command.is_blank() => {
                problems.push(Problem::new(format!("{at}.run: empty command")));
            }
            Ok(_) => {}
        }
    }
    problems
}
