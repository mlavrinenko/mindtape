#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::Path;

use super::{
    ChangeSet, DriftKind, InFlightBudget, build_exclude_matcher, check_estimate_drift,
    numstat_total, path_excluded, status_path, validate_exclude_globs, working_tree_changes,
};
use crate::config::{Config, DriftConfig, parse_config};
use crate::model::{Status, Task};
use crate::test_support::{bare_task, git_init};

/// A legend with one working and one terminal state, plus a `files` estimate
/// unit. Drift settings stay at their defaults: unit `files`, in-flight
/// `status == "wip"`, line multiple 200.
fn config() -> Config {
    parse_config(
        r#"
[[tasks.status]]
value = "wip"
[[tasks.status]]
value = "done"
terminal = true

[tasks.estimate]
units = ["files"]
"#,
    )
    .unwrap()
}

/// A task in `status`, declaring `count` files when `count` is `Some`.
fn task(id: &str, status: &str, count: Option<f64>) -> Task {
    Task {
        status: Some(Status {
            name: status.to_owned(),
            date: None,
            note: None,
        }),
        estimate: count.map_or_else(BTreeMap::new, |count| {
            BTreeMap::from([("files".to_owned(), count)])
        }),
        ..bare_task(id)
    }
}

/// The drift config the fixture [`config`] resolves to.
fn drift() -> DriftConfig {
    config().tasks.estimate.drift
}

#[test]
fn one_task_in_flight_carries_its_own_budget() {
    let tasks = [task("alpha", "wip", Some(3.0)), task("old", "done", None)];
    let budget = InFlightBudget::of(&tasks, &config()).unwrap().unwrap();
    assert_eq!(budget.tasks, vec!["alpha".to_owned()]);
    assert!((budget.files - 3.0).abs() < f64::EPSILON);
}

#[test]
fn several_in_flight_sum_their_budgets() {
    let tasks = [
        task("alpha", "wip", Some(3.0)),
        task("beta", "wip", Some(2.0)),
        task("gamma", "wip", None),
    ];
    let budget = InFlightBudget::of(&tasks, &config()).unwrap().unwrap();
    assert_eq!(
        budget.tasks.len(),
        3,
        "the whole in-flight set is attributed"
    );
    assert!((budget.files - 5.0).abs() < f64::EPSILON);
}

#[test]
fn nothing_in_flight_means_no_budget() {
    let tasks = [task("old", "done", Some(3.0))];
    assert!(InFlightBudget::of(&tasks, &config()).unwrap().is_none());
}

#[test]
fn an_in_flight_task_with_no_file_estimate_means_no_budget() {
    let tasks = [task("alpha", "wip", None)];
    assert!(
        InFlightBudget::of(&tasks, &config()).unwrap().is_none(),
        "with nothing declared there is nothing to compare against"
    );
}

#[test]
fn an_unusable_in_flight_filter_is_reported_not_guessed() {
    let mut config = config();
    config.tasks.estimate.drift.in_flight = "nonsense_field == 1".to_owned();
    assert!(InFlightBudget::of(&[task("alpha", "wip", Some(1.0))], &config).is_err());
}

/// A budget of `files` attributed to `count` tasks.
fn budget(files: f64, count: usize) -> InFlightBudget {
    InFlightBudget {
        tasks: (0..count).map(|n| format!("task-{n}")).collect(),
        files,
    }
}

#[test]
fn a_working_tree_inside_the_budget_is_silent() {
    let changes = ChangeSet {
        files: 3,
        lines: 40,
    };
    assert!(budget(5.0, 1).compare(changes, &drift()).is_empty());
}

#[test]
fn a_working_tree_at_the_budget_is_silent() {
    let changes = ChangeSet {
        files: 5,
        lines: 40,
    };
    assert!(
        budget(5.0, 1).compare(changes, &drift()).is_empty(),
        "the declared count is spendable in full before anything is said"
    );
}

#[test]
fn a_working_tree_over_the_budget_warns_and_names_the_single_task() {
    let changes = ChangeSet {
        files: 8,
        lines: 40,
    };
    let warnings = budget(5.0, 1).compare(changes, &drift());
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    let warning = warnings.first().unwrap();
    assert_eq!(warning.kind, DriftKind::Files);
    let text = warning.to_string();
    assert!(
        text.contains("working tree touches 8 file(s), over the 5 declared by `task-0`"),
        "{text}"
    );
    assert!(
        text.contains("split the remaining work into a new task"),
        "{text}"
    );
}

#[test]
fn several_in_flight_report_the_figure_as_a_total() {
    let changes = ChangeSet {
        files: 9,
        lines: 40,
    };
    let warnings = budget(6.0, 3).compare(changes, &drift());
    let text = warnings.first().unwrap().to_string();
    assert!(
        text.contains("declared in total by 3 in-flight tasks"),
        "several in flight must say the budget is a total: {text}"
    );
}

#[test]
fn the_line_warning_is_secondary_and_generous() {
    let inside = ChangeSet {
        files: 4,
        lines: 900,
    };
    assert!(
        budget(5.0, 1).compare(inside, &drift()).is_empty(),
        "900 lines against a 5-file budget (1000 allowed) is not yet a signal"
    );
    let over = ChangeSet {
        files: 4,
        lines: 1400,
    };
    let warnings = budget(5.0, 1).compare(over, &drift());
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    let warning = warnings.first().unwrap();
    assert_eq!(warning.kind, DriftKind::Lines);
    let text = warning.to_string();
    assert!(
        text.contains(
            "working tree changes 1400 line(s), over the 1000 implied by the file budget"
        ),
        "{text}"
    );
}

#[test]
fn a_zero_line_multiple_disables_the_line_warning() {
    let drift = DriftConfig {
        line_multiple: 0,
        ..DriftConfig::default()
    };
    let changes = ChangeSet {
        files: 2,
        lines: 90_000,
    };
    assert!(budget(5.0, 1).compare(changes, &drift).is_empty());
}

#[test]
fn both_budgets_can_break_at_once() {
    let changes = ChangeSet {
        files: 20,
        lines: 5000,
    };
    let warnings = budget(5.0, 2).compare(changes, &drift());
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert_eq!(warnings.first().unwrap().kind, DriftKind::Files);
    assert_eq!(warnings.get(1).unwrap().kind, DriftKind::Lines);
}

#[test]
fn numstat_sums_added_and_deleted_and_skips_binaries() {
    let text = "3\t4\tsrc/a.rs\n-\t-\tassets/logo.png\n10\t0\tsrc/b.rs\n";
    assert_eq!(numstat_total(text), 17);
}

#[test]
fn numstat_ignores_malformed_rows() {
    assert_eq!(numstat_total(""), 0);
    assert_eq!(numstat_total("garbage\n5\n"), 0);
}

#[test]
fn outside_a_repository_the_check_degrades_to_silence() {
    let missing = Path::new("/nonexistent-mindtape-drift-probe");
    assert!(
        working_tree_changes(missing, &DriftConfig::default()).is_none(),
        "a path git cannot read yields no change set, not a wrong one"
    );
}

#[test]
fn the_check_never_errors_on_a_breach() {
    let dir = tempfile::tempdir().unwrap();
    let tasks = [task("alpha", "wip", Some(1.0))];
    assert!(
        check_estimate_drift(&tasks, &config(), dir.path()).is_ok(),
        "drift is a warning: the check must not turn into a failure"
    );
}

// --- `[tasks.estimate.drift] exclude` -------------------------------------

#[test]
fn status_path_reads_the_path_off_an_untracked_or_modified_line() {
    assert_eq!(
        status_path("?? docs/generated/OUT.md"),
        Some("docs/generated/OUT.md")
    );
    assert_eq!(status_path(" M src/lib.rs"), Some("src/lib.rs"));
}

#[test]
fn status_path_resolves_a_rename_to_its_destination() {
    assert_eq!(
        status_path("R  old/name.rs -> new/name.rs"),
        Some("new/name.rs"),
        "a rename must count against the path the tree now holds, not the one it left"
    );
}

#[test]
fn status_path_strips_gits_quoting() {
    assert_eq!(
        status_path("?? \"a weird name.md\""),
        Some("a weird name.md")
    );
}

#[test]
fn status_path_is_none_on_a_line_too_short_to_carry_a_path() {
    assert_eq!(status_path("??"), None);
}

#[test]
fn path_excluded_matches_a_repo_root_relative_glob_crossing_directories() {
    let matcher = build_exclude_matcher(&["docs/generated/**".to_owned()]);
    assert!(path_excluded("?? docs/generated/OUT.md", matcher.as_ref()));
    assert!(
        path_excluded("?? docs/generated/nested/OUT.md", matcher.as_ref()),
        "`**` must cross directory boundaries"
    );
    assert!(!path_excluded("?? src/lib.rs", matcher.as_ref()));
}

#[test]
fn path_excluded_is_false_with_no_configured_pattern() {
    assert!(build_exclude_matcher(&[]).is_none());
    assert!(!path_excluded("?? docs/generated/OUT.md", None));
}

/// Real regression coverage for `truth/tasks/exclude-generated-outputs-from-
/// the-estimate-drift-check.typ`: a generated path sitting beside a
/// hand-edited one must not inflate the count `mt check` compares against the
/// declared budget. Runs against a real `git init`, not a filter over a
/// synthetic string, so it pins the exact behaviour `working_tree_changes`
/// hands back — not just that the option parses.
#[test]
fn working_tree_changes_excludes_a_generated_path_but_counts_the_real_one() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git_init(root);
    std::fs::create_dir_all(root.join("docs/generated")).unwrap();
    std::fs::write(root.join("docs/generated/OUT.md"), "generated\n").unwrap();
    std::fs::write(root.join("real.rs"), "fn main() {}\n").unwrap();

    let drift = DriftConfig {
        exclude: vec!["docs/generated/**".to_owned()],
        ..DriftConfig::default()
    };
    let changes = working_tree_changes(root, &drift).unwrap();
    assert_eq!(
        changes.files, 1,
        "only real.rs must count; the generated doc is excluded: {changes:?}"
    );

    let unfiltered = working_tree_changes(root, &DriftConfig::default()).unwrap();
    assert_eq!(
        unfiltered.files, 2,
        "with no exclude both untracked paths count — proving the exclude is what dropped the total"
    );
}

#[test]
fn a_valid_exclude_list_reports_no_problems() {
    let mut config = config();
    config.tasks.estimate.drift.exclude = vec!["docs/generated/**".to_owned()];
    assert!(validate_exclude_globs(&config).is_empty());
}

#[test]
fn an_invalid_exclude_glob_is_reported_not_silently_ignored() {
    let mut config = config();
    config.tasks.estimate.drift.exclude = vec!["[invalid".to_owned()];
    let problems = validate_exclude_globs(&config);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems
            .first()
            .unwrap()
            .contains("tasks.estimate.drift.exclude"),
        "{problems:?}"
    );
}
