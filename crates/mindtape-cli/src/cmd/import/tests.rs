//! Tests for `mt import`: the milestone round-trip, input shapes, `--dry-run`,
//! and the leave-no-stray-file guarantee on a failed record.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::link::Link;
use mindtape::model::numeric::NumValue;
use mindtape::model::{Status, Task};
use mindtape::test_support::write_file;
use mindtape::test_support::{bare_task, dep_edge, ice_priority, noted_edge};
use mindtape::{CanonicalTask, Config, load_project};
use typst_harvest::HVal;

use super::{ImportArgs, import_records};
use crate::util::Project;

use mindtape::test_support::PRELUDE;

/// `ice` as the priority framework (scalar priorities stay allowed — the
/// default) and an explicit `wip`/`done` legend, so the fixture can mix
/// framework and scalar priorities and exercise `--map`/`--status` against a
/// legend with no `proposed` entry.
const CONFIG_TEXT: &str = "[tasks.priority]\nframework = \"ice\"\n\
     [[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// A project rooted at `dir` with the in-tree `lib/` override, a real
/// `.mindtape/config.toml` (`land`/`preview` reload it from disk), and an
/// empty `tasks/` root — the import destination.
pub(super) fn setup_dest() -> (tempfile::TempDir, Project) {
    crate::test_util::make_project_with_lib(PRELUDE, &[], CONFIG_TEXT)
}

pub(super) fn bare_args() -> ImportArgs {
    ImportArgs {
        input: "-".to_owned(),
        into: None,
        to: None,
        dry_run: false,
        status: None,
        map: vec![],
        drop_unmapped: false,
    }
}

fn status(name: &str, date: &str) -> Status {
    Status {
        name: name.to_owned(),
        date: Some(date.to_owned()),
        note: None,
    }
}

/// A few tasks with framework priority, cross-links, tags, extras, and a
/// body — titles are chosen so their slug equals the id/filename exactly, so
/// re-importing mints the same stem and inbound `links:` targets resolve.
fn fixture_tasks() -> Vec<Task> {
    let mut alpha = bare_task("task-alpha");
    alpha.title = "Task Alpha".to_owned();
    alpha.status = Some(status("wip", "2026-06-01"));
    alpha.priority = Some(ice_priority(8.0, 0.75, 4.0));
    alpha.tags = vec!["backend".to_owned(), "cli".to_owned()];
    alpha.links = vec![dep_edge("task-beta.typ")];
    alpha.body = "== Summary\n\nDo the alpha thing.".to_owned();
    alpha.extras =
        std::collections::BTreeMap::from([("x-test".to_owned(), HVal::Str("hello".to_owned()))]);

    let mut beta = bare_task("task-beta");
    beta.title = "Task Beta".to_owned();
    beta.status = Some(status("done", "2026-05-20"));
    beta.priority = Some(NumValue::Num(5.0));
    beta.difficulty = Some(NumValue::Num(3.0));
    beta.due = Some("2026-07-01".to_owned());
    beta.body = "== Summary\n\nFinish beta.".to_owned();

    let mut gamma = bare_task("task-gamma");
    gamma.title = "Task Gamma".to_owned();
    gamma.status = Some(status("wip", "2026-06-10"));
    gamma.links = vec![noted_edge("task-alpha.typ", "related", "see also")];
    gamma.body = "== Summary\n\nGamma notes.".to_owned();

    vec![alpha, beta, gamma]
}

fn link_sort_key(link: &Link) -> (String, String, Option<String>) {
    (link.kind.clone(), link.task.clone(), link.note.clone())
}

pub(super) fn to_canonical(tasks: &[Task], config: &Config) -> String {
    let canonical: Vec<CanonicalTask> = tasks
        .iter()
        .map(|task| CanonicalTask::from_task(task, config))
        .collect();
    serde_json::to_string(&canonical).unwrap()
}

#[test]
fn round_trips_the_batch_field_for_field_including_body() {
    let (dir, project) = setup_dest();
    let fixtures = fixture_tasks();
    let raw = to_canonical(&fixtures, &project.config);

    let outcomes = import_records(&project, &bare_args(), &raw).unwrap();
    assert_eq!(outcomes.len(), 3, "{raw}");
    assert!(
        outcomes.iter().all(|outcome| outcome.filed),
        "{:?}",
        outcomes
            .iter()
            .map(|outcome| (&outcome.label, &outcome.problems))
            .collect::<Vec<_>>()
    );

    let reloaded = load_project(dir.path(), &project.config);
    assert!(reloaded.errors.is_empty(), "{:?}", reloaded.errors);

    for expected in &fixtures {
        let mut actual = reloaded
            .tasks
            .iter()
            .find(|task| task.id == expected.id)
            .unwrap_or_else(|| panic!("task {} did not land", expected.id))
            .clone();
        actual.file.clone_from(&expected.file);
        actual.derived_tags.clone_from(&expected.derived_tags);
        // Loader-derived from the whole set, like `derived_tags`, and written
        // to no file: `task-alpha` depends on `task-beta`, so reloading gives
        // it a graph position the authored fixture never carried. The import
        // schema round-trips authored fields; the ranks are not one.
        actual.topo = expected.topo;
        actual.blocks = expected.blocks;
        // Likewise `spans`: a property of the file the loader read, not
        // of the record the import schema carries.
        actual.spans = expected.spans.clone();
        actual.links.sort_by_key(link_sort_key);

        let mut expected = expected.clone();
        expected.links.sort_by_key(link_sort_key);
        assert_eq!(actual, expected, "task {} did not round-trip", expected.id);
    }
}

#[test]
fn bare_array_and_envelope_both_parse() {
    let (_dir, project) = setup_dest();
    let mut task = bare_task("solo");
    task.title = "Solo Task".to_owned();
    task.status = Some(status("wip", "2026-06-01"));
    let raw_array = to_canonical(&[task.clone()], &project.config);

    let outcomes = import_records(&project, &bare_args(), &raw_array).unwrap();
    assert_eq!(outcomes.len(), 1);
    assert!(outcomes[0].filed, "{:?}", outcomes[0].problems);

    // Re-run against the envelope shape into a second, independent project.
    let (_dir2, project2) = setup_dest();
    let canonical = mindtape::CanonicalTask::from_task(&task, &project2.config);
    let envelope = serde_json::json!({ "tasks": [canonical], "applied": {"rule": null} });
    let outcomes2 = import_records(&project2, &bare_args(), &envelope.to_string()).unwrap();
    assert_eq!(outcomes2.len(), 1);
    assert!(outcomes2[0].filed, "{:?}", outcomes2[0].problems);
}

#[test]
fn envelope_with_current_schema_version_imports() {
    let (_dir, project) = setup_dest();
    let mut task = bare_task("solo");
    task.title = "Solo Task".to_owned();
    task.status = Some(status("wip", "2026-06-01"));
    let canonical = mindtape::CanonicalTask::from_task(&task, &project.config);
    let envelope = serde_json::json!({
        "schema_version": mindtape::SCHEMA_VERSION,
        "tasks": [canonical],
    });

    let outcomes = import_records(&project, &bare_args(), &envelope.to_string()).unwrap();
    assert_eq!(outcomes.len(), 1);
    assert!(outcomes[0].filed, "{:?}", outcomes[0].problems);
}

#[test]
fn envelope_with_a_newer_schema_version_is_rejected() {
    let (_dir, project) = setup_dest();
    let too_new = mindtape::SCHEMA_VERSION + 1;
    let envelope = serde_json::json!({
        "schema_version": too_new,
        "tasks": [],
    });

    // `TaskOutcome` (the `Ok` payload) has no `Debug` impl, so match instead
    // of `unwrap_err` (which requires `T: Debug` for its own panic message).
    let err = match import_records(&project, &bare_args(), &envelope.to_string()) {
        Err(err) => crate::diag::plain(&err),
        Ok(_) => panic!("expected the newer schema_version to be rejected"),
    };
    assert!(err.contains(&too_new.to_string()), "{err}");
    assert!(err.contains(&mindtape::SCHEMA_VERSION.to_string()), "{err}");
}

#[test]
fn dry_run_writes_nothing_to_the_destination() {
    let (dir, project) = setup_dest();
    let mut task = bare_task("preview-me");
    task.title = "Preview Me".to_owned();
    task.status = Some(status("wip", "2026-06-01"));
    let raw = to_canonical(&[task], &project.config);

    let mut args = bare_args();
    args.dry_run = true;
    let outcomes = import_records(&project, &args, &raw).unwrap();
    assert_eq!(outcomes.len(), 1);
    assert!(outcomes[0].filed, "{:?}", outcomes[0].problems);

    assert!(
        !dir.path().join("tasks/preview-me.typ").exists(),
        "dry-run must not write to the real destination"
    );
    let entries: Vec<_> = std::fs::read_dir(dir.path().join("tasks"))
        .unwrap()
        .flatten()
        .collect();
    assert!(entries.is_empty(), "dry-run left files: {entries:?}");
}

#[test]
fn a_record_failing_validation_is_reported_and_leaves_no_file() {
    let (dir, project) = setup_dest();
    let mut bad = bare_task("bad-status");
    bad.title = "Bad Status".to_owned();
    // `blocked` is outside CONFIG_TEXT's `wip`/`done` legend and no
    // `--map`/`--status` fix is supplied, so this must fail post-write
    // validation and leave no file behind.
    bad.status = Some(status("blocked", "2026-06-01"));
    let raw = to_canonical(&[bad], &project.config);

    let outcomes = import_records(&project, &bare_args(), &raw).unwrap();
    assert_eq!(outcomes.len(), 1);
    assert!(!outcomes[0].filed, "expected the bad record to be skipped");
    assert!(!outcomes[0].problems.is_empty());

    assert!(!dir.path().join("tasks/bad-status.typ").exists());
    let entries: Vec<_> = std::fs::read_dir(dir.path().join("tasks"))
        .unwrap()
        .flatten()
        .collect();
    assert!(
        entries.is_empty(),
        "a failed record must not leave a stray file: {entries:?}"
    );
}

#[test]
fn status_default_flag_fills_a_missing_status() {
    let (dir, project) = setup_dest();
    let mut task = bare_task("needs-default");
    task.title = "Needs Default".to_owned();
    task.status = None;
    let raw = to_canonical(&[task], &project.config);

    let mut args = bare_args();
    args.status = Some("wip".to_owned());
    let outcomes = import_records(&project, &args, &raw).unwrap();
    assert!(outcomes[0].filed, "{:?}", outcomes[0].problems);

    let written = std::fs::read_to_string(dir.path().join("tasks/needs-default.typ")).unwrap();
    assert!(written.contains("status: wip("), "{written}");
}

#[test]
fn map_renames_a_foreign_status_before_legend_validation() {
    let (dir, project) = setup_dest();
    let mut task = bare_task("todo-task");
    task.title = "Todo Task".to_owned();
    task.status = Some(status("todo", "2026-06-01"));
    let raw = to_canonical(&[task], &project.config);

    let mut args = bare_args();
    args.map = vec!["todo=wip".to_owned()];
    let outcomes = import_records(&project, &args, &raw).unwrap();
    assert!(outcomes[0].filed, "{:?}", outcomes[0].problems);

    let written = std::fs::read_to_string(dir.path().join("tasks/todo-task.typ")).unwrap();
    assert!(written.contains("status: wip("), "{written}");
}

#[test]
fn colliding_titles_disambiguate_with_a_numeric_suffix() {
    let (dir, project) = setup_dest();
    let mut one = bare_task("dup-a");
    one.title = "Duplicate".to_owned();
    one.status = Some(status("wip", "2026-06-01"));
    let mut two = bare_task("dup-b");
    two.title = "Duplicate".to_owned();
    two.status = Some(status("wip", "2026-06-01"));
    // Records from a jq adapter carry no `id`, so the title names the file.
    let mut records: Vec<CanonicalTask> =
        serde_json::from_str(&to_canonical(&[one, two], &project.config)).unwrap();
    for record in &mut records {
        record.id.clear();
    }
    let raw = serde_json::to_string(&records).unwrap();

    let outcomes = import_records(&project, &bare_args(), &raw).unwrap();
    assert_eq!(outcomes.len(), 2);
    assert!(outcomes.iter().all(|outcome| outcome.filed));
    assert!(dir.path().join("tasks/duplicate.typ").exists());
    assert!(dir.path().join("tasks/duplicate-2.typ").exists());
}

#[test]
fn a_record_with_an_id_keeps_it_as_its_stem_so_links_to_it_resolve() {
    let (dir, project) = setup_dest();
    let mut target = bare_task("add-redis-client");
    target.title = "Add a Redis client to the app state".to_owned();
    target.status = Some(status("wip", "2026-06-01"));
    let mut linker = bare_task("throttle-logins");
    linker.title = "Throttle failed logins".to_owned();
    linker.status = Some(status("wip", "2026-06-01"));
    linker.links = vec![dep_edge("add-redis-client.typ")];
    let raw = to_canonical(&[linker, target], &project.config);

    let outcomes = import_records(&project, &bare_args(), &raw).unwrap();
    assert!(
        outcomes.iter().all(|outcome| outcome.filed),
        "{:?}",
        outcomes.iter().map(|o| &o.problems).collect::<Vec<_>>()
    );
    assert!(dir.path().join("tasks/add-redis-client.typ").exists());
    assert!(dir.path().join("tasks/throttle-logins.typ").exists());
}

#[test]
fn an_id_that_clashes_with_an_existing_file_is_disambiguated() {
    let (dir, project) = setup_dest();
    std::fs::create_dir_all(dir.path().join("tasks")).unwrap();
    write_file(dir.path(), "tasks/taken.typ", "");
    let mut task = bare_task("taken");
    task.title = "Something else".to_owned();
    task.status = Some(status("wip", "2026-06-01"));
    let raw = to_canonical(&[task], &project.config);

    let outcomes = import_records(&project, &bare_args(), &raw).unwrap();
    assert!(outcomes[0].filed, "{:?}", outcomes[0].problems);
    assert!(dir.path().join("tasks/taken-2.typ").exists());
}

#[test]
fn a_title_that_slugifies_to_nothing_is_reported_and_leaves_no_file() {
    let (dir, project) = setup_dest();
    let mut punctuation_only = bare_task("");
    // Every character here is filtered by `slug_from_title`, so the record has
    // a non-empty title yet mints an empty stem candidate — `stem_for`'s own
    // guard, distinct from a record missing a title outright.
    punctuation_only.title = "!!! ---".to_owned();
    punctuation_only.status = Some(status("wip", "2026-06-01"));
    let raw = to_canonical(&[punctuation_only], &project.config);

    let outcomes = import_records(&project, &bare_args(), &raw).unwrap();
    assert_eq!(outcomes.len(), 1);
    assert!(
        !outcomes[0].filed,
        "a title with no alphanumerics must not be filed"
    );
    assert!(
        outcomes[0]
            .problems
            .iter()
            .any(|problem| problem.contains("empty title")),
        "{:?}",
        outcomes[0].problems
    );
    let entries: Vec<_> = std::fs::read_dir(dir.path().join("tasks"))
        .unwrap()
        .flatten()
        .collect();
    assert!(entries.is_empty(), "no file must land: {entries:?}");
}

#[test]
fn a_malformed_map_entry_fails_the_batch_before_any_record_is_written() {
    let (dir, project) = setup_dest();
    let mut task = bare_task("solo");
    task.title = "Solo Task".to_owned();
    task.status = Some(status("wip", "2026-06-01"));
    let raw = to_canonical(&[task], &project.config);

    let mut args = bare_args();
    args.map = vec!["=bad".to_owned()];
    let err = match import_records(&project, &args, &raw) {
        Err(err) => err.to_string(),
        Ok(outcomes) => panic!(
            "expected the malformed --map entry to be rejected, got {} outcomes",
            outcomes.len()
        ),
    };
    assert!(err.contains("OLD=NEW"), "{err}");
    assert!(
        !dir.path().join("tasks/solo-task.typ").exists(),
        "a rejected --map must never reach the write stage"
    );
}
