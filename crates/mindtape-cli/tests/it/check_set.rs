//! `mt @set check` checks every project of a set and reports them together.
#![allow(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing
)]

use std::path::Path;

use crate::at_e2e::{run, setup};
use crate::support;

fn json_of(out: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap()
}

fn project_names(json: &serde_json::Value) -> Vec<&str> {
    let projects = json["projects"].as_array().unwrap();
    projects
        .iter()
        .map(|p| p["project"].as_str().unwrap())
        .collect()
}

/// The stderr of a run that exits 1.
fn fails(args: &[&str], cfg: &tempfile::TempDir) -> String {
    let out = run(cfg.path(), args);
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(out.status.code(), Some(1), "{stderr}");
    stderr
}

fn break_config(project: &Path) {
    let path = project.join(".mindtape/config.toml");
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str("\n[tasks.estimate.units.nope]\nmax = 3\n");
    std::fs::write(path, text).unwrap();
}

#[test]
fn a_clean_set_ends_with_one_ok_line() {
    let (_alpha, _beta, cfg) = setup();
    let out = run(cfg.path(), &["@both", "check"]);
    let (ok, text) = support::text_of(&out);
    assert!(ok, "{text}");
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "ok: 2 projects, 2 tasks\n"
    );
}

#[test]
fn a_broken_member_is_reported_by_name_and_fails_the_run() {
    let (_alpha, beta, cfg) = setup();
    break_config(beta.path());
    let stderr = fails(&["@both", "check"], &cfg);
    let lines: Vec<&str> = stderr.lines().collect();
    assert!(
        lines.iter().any(|l| l.contains("can't parse config")),
        "{stderr}"
    );
    assert!(
        lines.iter().any(|l| l.contains("note: project: ")),
        "{stderr}"
    );
    assert_eq!(lines.last(), Some(&"error: 1 of 2 projects failed"));
}

#[test]
fn json_prints_one_object_per_project() {
    let (alpha, beta, cfg) = setup();
    break_config(beta.path());
    let out = run(cfg.path(), &["@both", "check", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["schema_version"], mindtape::SCHEMA_VERSION);
    let projects = json["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 2);
    let name = |dir: &Path| dir.file_name().unwrap().to_str().unwrap().to_owned();
    assert_eq!(projects[0]["project"], name(alpha.path()));
    assert_eq!(projects[0]["ok"], true);
    assert_eq!(projects[0]["problems"], serde_json::json!([]));
    assert_eq!(projects[1]["project"], name(beta.path()));
    assert_eq!(projects[1]["ok"], false);
    assert_eq!(projects[1]["problems"][0]["severity"], "error");
}

#[test]
fn the_commit_flags_refuse_a_set() {
    let (_alpha, _beta, cfg) = setup();
    for args in [["--commits", "main..HEAD"], ["--commit-msg", "MSG"]] {
        let stderr = fails(&["@both", "check", args[0], args[1]], &cfg);
        assert!(
            stderr.contains("runs on one project, and @both holds 2 projects"),
            "{stderr}"
        );
    }
}

#[test]
fn reports_keep_set_order_though_the_checks_run_in_parallel() {
    let many: Vec<(String, &str)> = (0..40).map(|n| (format!("t{n}.typ"), "Task")).collect();
    let slow = support::project_with_tasks(
        &many
            .iter()
            .map(|(f, t)| (f.as_str(), *t))
            .collect::<Vec<_>>(),
    );
    let quick: Vec<_> = (0..5).map(|_| support::project_with_tasks(&[])).collect();
    let toml = |dir: &Path| dir.display().to_string().replace('\\', "/");
    let paths: Vec<String> = std::iter::once(&slow)
        .chain(&quick)
        .map(|dir| format!("\"{}\"", toml(dir.path())))
        .collect();
    let cfg = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(cfg.path().join("mindtape")).unwrap();
    std::fs::write(
        cfg.path().join("mindtape/at.toml"),
        format!("[at.many]\ninclude = [{}]\n", paths.join(", ")),
    )
    .unwrap();
    let out = run(cfg.path(), &["@many", "check", "--json"]);
    let json = json_of(&out);
    let names = project_names(&json);
    let expected: Vec<String> = std::iter::once(&slow)
        .chain(&quick)
        .map(|dir| dir.path().file_name().unwrap().to_str().unwrap().to_owned())
        .collect();
    assert_eq!(names, expected);
}

#[test]
fn a_task_rule_breach_fails_its_project_though_the_config_loads() {
    let (_alpha, beta, cfg) = setup();
    let path = beta.path().join(".mindtape/config.toml");
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str(
        "\n[[tasks.rules]]\nname = \"never\"\nassert = 'title == \"nothing\"'\n\
         message = \"no title passes\"\nseverity = \"error\"\n",
    );
    std::fs::write(path, text).unwrap();
    let out = run(cfg.path(), &["@both", "check", "--json"]);
    let json = json_of(&out);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(json["projects"][0]["ok"], true);
    assert_eq!(json["projects"][1]["ok"], false);
}
