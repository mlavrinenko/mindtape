//! Pins what `skills/mindtape/evals/evals.json` asserts about the api-service
//! fixture. The evals name its counts and its order by hand, and no model runs
//! in CI, so nothing else notices when the fixture drifts from the claim.
//!
//! The clock comes from `www/capture.sh`, the one place that declares it, so
//! this test reads the same "today" as every docs capture that runs the
//! fixture.
#![allow(clippy::unwrap_used)]

use crate::support;

use std::path::{Path, PathBuf};

/// The api-service fixture, in place: every command below is a read.
fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/api-service")
}

/// The fixture clock `www/capture.sh` declares as `API_SERVICE_NOW=<date>`.
fn fixture_clock() -> String {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../www/capture.sh");
    std::fs::read_to_string(script)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("API_SERVICE_NOW="))
        .expect("www/capture.sh must declare API_SERVICE_NOW")
        .to_owned()
}

/// The task ids `mt ls <filter> --sort priority:desc --json` prints, in order.
fn ids(filter: &str) -> Vec<String> {
    let output = support::mt(&fixture())
        .env("MINDTAPE_NOW", fixture_clock())
        .args(["ls", filter, "--sort", "priority:desc", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success(), "mt ls {filter:?} failed");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    json.get("tasks")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|task| {
            task.get("id")
                .and_then(serde_json::Value::as_str)
                .unwrap()
                .to_owned()
        })
        .collect()
}

const UPDATE: &str =
    "update skills/mindtape/evals/evals.json (evals 0-3) and its README.md in the same commit";

#[test]
fn the_fixture_has_the_task_counts_the_evals_state() {
    assert_eq!(
        ids("").len(),
        12,
        "evals 1 and 3 say api-service has 12 tasks; {UPDATE}"
    );
    assert_eq!(
        ids("terminal").len(),
        4,
        "eval 3 says 4 of them are terminal; {UPDATE}"
    );
}

#[test]
fn the_fixture_ready_list_matches_eval_zero() {
    let ready = ids("ready");
    assert_eq!(
        ready.len(),
        7,
        "eval 0 says seven tasks are ready; {UPDATE}"
    );
    assert_eq!(
        ready.first().map(String::as_str),
        Some("index-the-orders-by-customer"),
        "eval 0 says this task leads the ready list by priority; {UPDATE}"
    );
    assert!(
        !ready.iter().any(|id| id == "throttle-failed-logins-per-ip"),
        "eval 0 says throttle-failed-logins-per-ip is blocked, so not ready; {UPDATE}"
    );
}
