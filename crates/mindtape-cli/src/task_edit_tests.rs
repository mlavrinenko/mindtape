#![allow(clippy::unwrap_used)]

use tempfile::TempDir;

use super::verify_written;

const PRELUDE: &str = include_str!("../../../lib/prelude.typ");
const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n";

/// Scaffold a project holding a single `tasks/<name>` file and return the temp
/// dir plus the absolute path of that file, ready for [`verify_written`].
fn project_with(name: &str, body: &str) -> (TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    crate::test_util::make_project_with_lib(dir.path(), PRELUDE, &[(name, body)], CONFIG);
    let file = dir.path().join("tasks").join(name);
    (dir, file)
}

/// A task that harvests cleanly and validates: [`verify_written`] returns `Ok`.
/// Guards against a mutant that always bails.
#[test]
fn verify_written_accepts_a_valid_task() {
    let (dir, file) = project_with(
        "good.typ",
        "#import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(title: \"Good\", status: wip(2026, 1, 1), priority: 3)\n",
    );
    verify_written(dir.path(), &file).unwrap();
}

/// A file that fails to load (imports a package that does not exist) must be
/// reported as a load failure. Kills `verify_written -> Ok(())` and the `==`
/// on the error-lookup line: flipping it to `!=` would miss the on-`rel` error
/// and fall through to the "did not harvest" branch, so we pin the message.
#[test]
fn verify_written_reports_a_load_error_on_the_file() {
    let (dir, file) = project_with("broken.typ", "#import \"@local/nope:9.9.9\": *\n");
    let err = verify_written(dir.path(), &file).unwrap_err();
    assert!(
        err.to_string().contains("failed to load"),
        "expected a load-failure message, got: {err}"
    );
}

/// A task that harvests but violates the priority scale (max 9) must be
/// reported. Kills `verify_written -> Ok(())` and the `==` on the
/// violation filter: flipping it to `!=` keeps only *other* tasks' violations
/// — empty here — so the mutant would wrongly return `Ok`.
#[test]
fn verify_written_reports_a_validation_violation_on_the_file() {
    let (dir, file) = project_with(
        "outofscale.typ",
        "#import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(title: \"Out of scale\", status: wip(2026, 1, 1), priority: 99)\n",
    );
    let err = verify_written(dir.path(), &file).unwrap_err();
    assert!(
        err.to_string().contains("validation issue"),
        "expected a validation-issue message, got: {err}"
    );
}
