//! The scalar `mt add` flags, each of which is one `--set field=value` write.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

/// One flag, its value, the literal the filed task must carry, and a stem.
const CASES: &[(&str, &str, &str, &str)] = &[
    ("--due", "2026-12-01", "due: on(2026, 12, 1)", "dated"),
    ("--start", "2026-11-01", "start: on(2026, 11, 1)", "started"),
    ("--assignee", "ada", "assignee: \"ada\"", "assigned"),
    ("--kind", "bug", "kind: \"bug\"", "kinded"),
    ("--priority", "6", "priority: 6", "prioritised"),
    ("--difficulty", "3", "difficulty: 3", "difficult"),
];

#[test]
fn every_scalar_flag_writes_its_field() {
    for (flag, value, expected, name) in CASES {
        let dir = support::scaffold();
        support::run_ok(dir.path(), &["add", "--name", name, flag, value, "Flagged"]);
        let written =
            std::fs::read_to_string(dir.path().join("tasks").join(format!("{name}.typ"))).unwrap();
        assert!(written.contains(expected), "{flag}: {written}");
    }
}

/// `status` has its own add flag, so the generic writer's `mt flip` hint would
/// name a task that does not exist yet.
#[test]
fn a_status_token_hints_the_add_flag_not_the_flip_command() {
    let dir = support::scaffold();

    let (ok, text) = support::run_text(dir.path(), &["add", "--set", "status=wip", "T"]);
    assert!(!ok, "{text}");
    assert!(text.contains("--status"), "{text}");
    assert!(!text.contains("mt flip"), "{text}");
}

#[test]
fn a_flag_and_set_naming_one_field_is_refused() {
    for (flag, value, _, name) in CASES {
        let dir = support::scaffold();
        let field = flag.trim_start_matches("--");
        let output = support::mt(dir.path())
            .args([
                "add",
                "--name",
                name,
                flag,
                value,
                "--set",
                &format!("{field}={value}"),
                "Twice",
            ])
            .output()
            .unwrap();
        assert!(!output.status.success(), "{flag} should conflict");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("set twice"), "{flag}: {stderr}");
        assert!(stderr.contains(field), "{flag}: {stderr}");
    }
}
