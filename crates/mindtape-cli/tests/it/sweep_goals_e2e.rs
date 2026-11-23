//! The advisory sweep rates every goal the project declares in each task it
//! files. `contrib/ask-refactoring/goals.sh` builds that `== Goals` block from
//! the two registries rather than from a list of its own. The block fills a
//! required slot on every sweep template, so a goal the block omits fails the
//! arm's `mt add`. A list written beside the arms once broke every arm at a
//! rename, and nothing said so until a sweep ran.
//!
//! `the_goal_block_rates_each_registry_goal_once` reads the names the way the
//! sweep does and checks the block against them: one `#Helps`, the rest
//! `#Neutral`, no goal missing and none rated twice. `an_unknown_goal_refuses`
//! pins the other half: an arm whose goal a registry does not declare fails
//! before it can file a task.
//!
//! Unix only: the derivation runs `typst query` and `jq` from the dev shell.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support::{checkout, shell};
use mindtape::test_support::tempdir;
use std::process::Output;

/// The goal every sweep arm helps: its block rates it `#Helps` and leaves it
/// off the neutral line.
const HELPED: &str = "keep-changes-safe";

/// A goal only `truth/goals.typ` declares. `HELPED` comes only from
/// `truth/process-goals.typ`, so a driver that read one registry still agrees
/// with its own output, and these two catch it.
const PRODUCT_ONLY: &str = "read-tasks-at-a-glance";

/// Every query below sources the sweep's own helper from the checkout `$REPO`
/// names. The test then reads the file the sweep runs.
const SOURCE: &str = ". \"$REPO/contrib/ask-refactoring/goals.sh\"\n";

/// `script` run from a throwaway directory with the checkout in `$REPO`.
fn run(script: &str) -> Output {
    let dir = tempdir();
    shell("bash", dir.path())
        .args(["-eo", "pipefail", "-c", script])
        .env("REPO", checkout())
        .output()
        .unwrap()
}

/// What `script` printed, once it exits zero.
fn stdout(script: &str) -> String {
    let out = run(script);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{script} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// The goal names, in registry order, exactly as the sweep derives them.
fn goal_names() -> Vec<String> {
    stdout(&format!("{SOURCE}sweep_goal_names"))
        .lines()
        .map(str::to_owned)
        .collect()
}

/// The block rates `names` exactly once each: one help, every other goal
/// neutral, in registry order and with no repeat.
fn assert_rates_each_once(block: &str, names: &[String]) {
    let mut lines = block.lines();
    assert_eq!(lines.next(), Some("== Goals"));
    assert_eq!(lines.next(), Some(""));
    let helps = format!("+ #Helps(maintainer, to: {HELPED})[because the test says so].");
    assert_eq!(lines.next(), Some(helps.as_str()));
    let neutral = lines
        .next()
        .expect("the block rates the goals it does not help")
        .strip_prefix("+ #Neutral(")
        .and_then(|rest| rest.strip_suffix(")."))
        .unwrap_or_else(|| panic!("not a `#Neutral` line: {block}"));
    assert!(
        lines.next().is_none(),
        "the block has a line past `#Neutral`: {block}"
    );
    let rated: Vec<&str> = neutral.split(", ").collect();
    let expected: Vec<&str> = names
        .iter()
        .map(String::as_str)
        .filter(|name| *name != HELPED)
        .collect();
    assert_eq!(rated, expected);
}

#[test]
fn the_goal_block_rates_each_registry_goal_once() {
    let names = goal_names();
    assert!(
        names.len() > 1,
        "the registries declare {} goal(s)",
        names.len()
    );
    let mut unique = names.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(
        unique.len(),
        names.len(),
        "a goal is listed twice: {names:?}"
    );
    for wanted in [PRODUCT_ONLY, HELPED] {
        assert!(
            names.iter().any(|name| name == wanted),
            "the registries declare {wanted}, and the block would omit it: {names:?}"
        );
    }
    let block = stdout(&format!(
        "{SOURCE}goals_with_one_helps maintainer {HELPED} 'because the test says so'"
    ));
    assert_rates_each_once(&block, &names);
}

#[test]
fn an_unknown_goal_refuses() {
    let out = run(&format!(
        "{SOURCE}goals_with_one_helps maintainer keep-change-safe 'because the test says so'"
    ));
    assert!(
        !out.status.success(),
        "a goal no registry declares was accepted"
    );
    assert!(
        out.stdout.is_empty(),
        "a refused goal still printed a block: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains(
            "'keep-change-safe' is not a goal in truth/goals.typ or truth/process-goals.typ"
        ),
        "the refusal does not name the registries: {stderr}"
    );
}
