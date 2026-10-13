#![allow(clippy::unwrap_used)]

use tempfile::TempDir;
use typst_edit::apply;

use super::{task_with, upsert_named_arg, verify_written};

use mindtape::test_support::PRELUDE;
const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n";
const IMPORT: &str = mindtape::typst_import!("");

/// Scaffold a project holding a single `tasks/<name>` file and return the temp
/// dir plus the absolute path of that file, ready for [`verify_written`].
fn project_with(name: &str, body: &str) -> (TempDir, std::path::PathBuf) {
    let (dir, _) = crate::test_util::make_project_with_lib(PRELUDE, &[(name, body)], CONFIG);
    let file = dir.path().join("tasks").join(name);
    (dir, file)
}

/// Upsert each `(name, value)` into `source`'s `task.with(…)` call and splice
/// the whole set at once — the shape every write path builds, so what these
/// tests read is what would reach the file. All the edits are computed against
/// the same untouched `source`, which is what makes several insertions sharing
/// one anchor worth pinning.
fn upsert(source: &str, args: &[(&str, &str)]) -> String {
    let call = task_with(source).unwrap();
    let edits = args
        .iter()
        .map(|(name, value)| upsert_named_arg(source, &call, name, value).unwrap())
        .collect();
    apply(source, edits).unwrap()
}

/// Assert that `source` is a task file that loads and validates, by writing it
/// into a throwaway project — the one property a separator bug destroys, and
/// the one an assertion on the rendered text alone cannot see.
fn assert_loads(name: &str, source: &str) {
    let (dir, file) = project_with(name, source);
    verify_written(dir.path(), &file).unwrap();
}

/// The bug this seam was rewritten for: a `task.with(…)` call written on ONE
/// line ends with no separator before its closing paren, and an injected
/// `status:` used to land glued to the previous value —
/// `title: "One line"status: wip(…)` — a file that cannot parse, written
/// before the error surfaced. Legal Typst in, loadable Typst out.
#[test]
fn injects_into_a_single_line_call() {
    let out = upsert(
        &format!("{IMPORT}#show: task.with(title: \"One line\")\n"),
        &[("status", "wip(2026, 1, 1)")],
    );
    assert_eq!(
        out,
        format!("{IMPORT}#show: task.with(title: \"One line\", status: wip(2026, 1, 1))\n")
    );
    assert_loads("one-line.typ", &out);
}

/// The same missing separator one line down: a multi-line call whose last
/// argument carries no trailing comma. A newline is not an argument separator
/// in Typst, so the injected argument needs the comma just as much as the
/// single-line shape does — it merely looked plausible.
#[test]
fn injects_into_a_multi_line_call_with_no_trailing_comma() {
    let out = upsert(
        &format!("{IMPORT}\n#show: task.with(\n  title: \"No comma\"\n)\n"),
        &[("status", "wip(2026, 1, 1)")],
    );
    assert_eq!(
        out,
        format!(
            "{IMPORT}\n#show: task.with(\n  title: \"No comma\",\n  status: wip(2026, 1, 1)\n)\n"
        )
    );
    assert_loads("no-comma.typ", &out);
}

/// The shape every in-tree template is written in — multi-line, trailing comma
/// — is byte-identical to what the old anchor produced. The fix moved where the
/// insertion lands, so the case that always worked is pinned here rather than
/// left to be noticed as churn in a task file's diff.
#[test]
fn injects_into_a_multi_line_call_unchanged() {
    let out = upsert(
        &format!("{IMPORT}\n#show: task.with(\n  title: \"Multi\",\n)\n"),
        &[("status", "wip(2026, 1, 1)")],
    );
    assert_eq!(
        out,
        format!(
            "{IMPORT}\n#show: task.with(\n  title: \"Multi\",\n  status: wip(2026, 1, 1),\n)\n"
        )
    );
    assert_loads("multi.typ", &out);
}

/// An EMPTY argument list is the mirror case: nothing precedes the injected
/// argument, so it must take NO leading comma. `task.with(,status: …)` would be
/// as unparseable as the glued form this fix removes.
#[test]
fn injects_into_an_empty_argument_list_without_a_leading_comma() {
    let out = upsert(
        &format!("{IMPORT}#show: task.with()\n"),
        &[("status", "wip(2026, 1, 1)"), ("title", "\"Empty\"")],
    );
    assert!(!out.contains("with(,"), "no leading comma: {out}");
    assert!(out.contains("status: wip(2026, 1, 1)"), "{out}");
    assert!(out.contains("title: \"Empty\""), "{out}");
    assert_loads("empty-args.typ", &out);
}

/// Two insertions computed against the same source share one anchor, so each
/// carries its own separator or the second glues onto the first. `mt add`
/// injects `status` and `id` exactly this way, in one splice.
#[test]
fn several_insertions_are_each_separated() {
    let out = upsert(
        &format!("{IMPORT}#show: task.with(title: \"Two\")\n"),
        &[("status", "wip(2026, 1, 1)"), ("priority", "3")],
    );
    assert_eq!(
        out,
        format!("{IMPORT}#show: task.with(title: \"Two\", status: wip(2026, 1, 1), priority: 3)\n")
    );
    assert_loads("two-inserts.typ", &out);
}

/// An argument the call already carries is replaced in place, separators and
/// all — the path the insertion rules must not disturb.
#[test]
fn replaces_an_existing_argument_in_place() {
    let out = upsert(
        &format!("{IMPORT}\n#show: task.with(\n  title: \"Old\",\n  status: wip(2026, 1, 1),\n)\n"),
        &[("title", "\"New\"")],
    );
    assert_eq!(
        out,
        format!("{IMPORT}\n#show: task.with(\n  title: \"New\",\n  status: wip(2026, 1, 1),\n)\n")
    );
    assert_loads("replaced.typ", &out);
}

/// A task that harvests cleanly and validates: [`verify_written`] returns `Ok`.
/// Guards against a mutant that always bails.
#[test]
fn verify_written_accepts_a_valid_task() {
    let (dir, file) = project_with(
        "good.typ",
        mindtape::typst_task!("(title: \"Good\", status: wip(2026, 1, 1), priority: 3)\n"),
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

/// A project whose config carries an `imperative-title` rule at `severity`
/// plus one task that breaches it, under the in-tree prelude.
fn rules_project(severity: &str) -> (TempDir, std::path::PathBuf) {
    let config = format!(
        "{CONFIG}\n[[tasks.rules]]\n\
         name = \"imperative-title\"\n\
         assert = 'matches(title, \"^Refactor\")'\n\
         message = \"start the title with an imperative verb\"\n\
         severity = \"{severity}\"\n"
    );
    let body = format!(
        "{IMPORT}#show: task.with(title: \"Loader refactoring\", status: wip(2026, 1, 1))\n"
    );
    let (dir, _) = crate::test_util::make_project_with_lib(PRELUDE, &[("bad.typ", &body)], &config);
    let file = dir.path().join("tasks/bad.typ");
    (dir, file)
}

/// The post-write fallback prints a fatal violation as `error:` and a
/// `severity = "warn"` breach as `warning:` — the same distinction the
/// pre-write seam makes, so the per-issue lines match what the caller does
/// with the write. `stderr` is only observable from a fresh process, so the
/// test re-execs itself.
#[test]
fn verify_written_prefixes_each_issue_by_its_severity() {
    const CHILD: &str = "_MT_VERIFY_WRITTEN_SEVERITY";
    if let Some(out) = mindtape::test_support::rerun_isolated(CHILD) {
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("error: ")
                && stderr.contains("bad.typ:")
                && stderr.contains(": rules.imperative-title"),
            "a fatal rule breach prints as an error: {stderr}"
        );
        assert!(
            stderr.contains("warning: ")
                && stderr.contains("bad.typ:")
                && stderr.contains(": rules.imperative-title"),
            "a warn rule breach prints as a warning: {stderr}"
        );
        return;
    }

    let (dir, file) = rules_project("error");
    // The refusal is the command's error, printed by `main`.
    let err = verify_written(dir.path(), &file).unwrap_err();
    crate::diag::emit(&err);
    let (dir, file) = rules_project("warn");
    verify_written(dir.path(), &file).unwrap();
}

/// A task that harvests but violates the priority scale (max 9) must be
/// reported. Kills `verify_written -> Ok(())` and the `==` on the
/// violation filter: flipping it to `!=` keeps only *other* tasks' violations
/// — empty here — so the mutant would wrongly return `Ok`.
#[test]
fn verify_written_reports_a_validation_violation_on_the_file() {
    let (dir, file) = project_with(
        "outofscale.typ",
        mindtape::typst_task!("(title: \"Out of scale\", status: wip(2026, 1, 1), priority: 99)\n"),
    );
    let err = verify_written(dir.path(), &file).unwrap_err();
    assert!(
        err.to_string().contains("priority") && !err.to_string().contains("validation issue"),
        "expected the breach itself, with no summary, got: {err}"
    );
}

fn filing(template: &str, explicit: Option<&str>) -> Option<String> {
    let config = mindtape::parse_config(CONFIG).unwrap();
    super::filing_status(&config, template, explicit)
}

#[test]
fn filing_status_reads_the_template_s_status_shortcut() {
    let template = "#show: task.with(title: slot(\"t\"), status: done(2026, 1, 1)[why])\n";
    assert_eq!(filing(template, None).as_deref(), Some("done"));
    assert_eq!(filing(template, Some("wip")).as_deref(), Some("wip"));
    let generic = "#show: task.with(status: status(\"shelved\", 2026, 1, 1))\n";
    assert_eq!(filing(generic, None).as_deref(), Some("shelved"));
}

#[test]
fn filing_status_falls_back_to_the_initial_status() {
    let initial = filing("#show: task.with(title: \"t\")\n", None);
    assert!(initial.is_some());
    for status in ["\"done\"", "slot(\"status\")", "(name: \"done\")"] {
        let template = format!("#show: task.with(status: {status})\n");
        assert_eq!(filing(&template, None), initial, "{status}");
    }
}
