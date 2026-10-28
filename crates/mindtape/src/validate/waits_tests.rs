#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::check_all;
use crate::config::Config;
use crate::test_support::bare_task;
use crate::validate::Violation;
use crate::waits::Checker;

fn config() -> Config {
    let mut config = Config::default();
    config.tasks.waits = vec![Checker {
        name: "up".to_owned(),
        run: crate::shell::ConfigCommand::Line("exit 0".to_owned()),
    }];
    config
}

fn breaches(entries: &[&str]) -> Vec<Violation> {
    let mut task = bare_task("t");
    task.waits_on = entries.iter().map(|&entry| entry.to_owned()).collect();
    let mut out = Vec::new();
    check_all(&task, &config(), &mut out);
    out
}

#[test]
fn a_runnable_entry_is_clean() {
    assert!(breaches(&["up:pr-7"]).is_empty());
}

#[test]
fn each_bad_entry_is_named_by_its_index() {
    let out = breaches(&["up:pr-7", "upp:pr-7", "up:x;y"]);

    let fields: Vec<&str> = out.iter().map(|breach| breach.field.as_str()).collect();
    assert_eq!(fields, ["waits-on[1]", "waits-on[2]"]);
    assert_eq!(out[0].message, "unknown checker: 'upp'");
    assert_eq!(
        out[0].unknown.as_ref().unwrap().nearest.as_deref(),
        Some("up")
    );
    assert!(out[1].unknown.is_none());
}
