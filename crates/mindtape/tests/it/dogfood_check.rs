//! The dogfood project (this repo's own `.mindtape/config.toml` +
//! `truth/tasks/*.typ`) must pass full task validation, not just the
//! JSON-schema shape check `config_schema_validate.rs` runs — otherwise a
//! required field like `estimate` could be flipped on in config while the
//! actual backlog silently fails only via the git pre-commit hook, never
//! `cargo test`.
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

fn root() -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "..", ".."].iter().collect()
}

/// Loaded with both environment-derived directories dropped: through
/// `load_config` this would merge the global `defaults.toml` of whoever runs
/// the suite, and a `[tasks]` line there could fail the repo's own backlog on
/// their machine alone.
fn config() -> mindtape::Config {
    mindtape::load_config_layered_from(&root(), None, None)
        .unwrap()
        .0
}

#[test]
fn dogfood_truth_tasks_pass_validation() {
    let root = root();
    let config = config();
    let outcome = mindtape::load_project(&root, &config);
    assert!(outcome.errors.is_empty(), "{:#?}", outcome.errors);

    // Only what fails `mt check` counts: a warn rule such as `src-budget`
    // reads the git working tree, so it fires on whatever is in flight.
    let violations: Vec<_> = mindtape::validate(&outcome.tasks, &config)
        .into_iter()
        .filter(mindtape::Violation::is_fatal)
        .collect();
    assert!(violations.is_empty(), "{violations:#?}");
}

/// `kind` names the kind of work a task is. The dogfood config declares the
/// vocabulary, so a misspelled or invented kind fails `mt check` rather than
/// silently falling out of every `kind == "milestone"` filter (the WIP limits).
#[test]
fn dogfood_config_rejects_an_unknown_kind() {
    let root = root();
    let config = config();
    let outcome = mindtape::load_project(&root, &config);
    let mut item = outcome.tasks.first().unwrap().clone();

    for known in KINDS {
        item.kind = Some(known.to_owned());
        let violations = mindtape::validate(std::slice::from_ref(&item), &config);
        assert!(
            !violations.iter().any(|v| v.field == "kind"),
            "{known}: {violations:#?}"
        );
    }

    // `task` says nothing a task file does not already say; `chore` and
    // `umbrella` are the retired spellings of `dev` and `milestone`.
    for unknown in ["epic", "task", "chore", "umbrella"] {
        item.kind = Some(unknown.to_owned());
        let violations = mindtape::validate(std::slice::from_ref(&item), &config);
        assert!(
            violations.iter().any(|v| v.field == "kind"),
            "{unknown} must be flagged: {violations:#?}"
        );
    }
}

/// The commit types a task closes with — `dev` for the harness around the
/// repo — plus `milestone`, which closes through its leaves rather than a
/// commit of its own.
const KINDS: [&str; 7] = [
    "feat",
    "fix",
    "refactor",
    "docs",
    "test",
    "dev",
    "milestone",
];

/// An open task names its kind; a closed one is exempt, so history is not
/// backfilled.
///
/// Both halves are stamped onto a loaded task rather than searched for in the
/// backlog: a backlog whose every task is closed is a legitimate state — the
/// one this test used to unwrap a `None` on — and the assertion is about the
/// config's `required` rule, not about what the backlog holds today.
#[test]
fn dogfood_config_requires_kind_on_open_tasks_only() {
    let root = root();
    let config = config();
    let outcome = mindtape::load_project(&root, &config);
    let base = outcome.tasks.first().unwrap().clone();
    let kind_flagged = |status: &str| {
        let mut item = base.clone();
        item.kind = None;
        item.status = Some(mindtape::Status {
            name: status.to_owned(),
            date: base.status.as_ref().and_then(|stamp| stamp.date.clone()),
            note: None,
        });
        mindtape::validate(&[item], &config)
            .iter()
            .any(|v| v.field == "kind")
    };

    let open = config.tasks.status.initial().unwrap().value.clone();
    let closed = config.tasks.status.terminal().first().unwrap().clone();
    assert!(kind_flagged(&open), "an open task without kind");
    assert!(!kind_flagged(&closed), "a closed task without kind");
}

/// A sweep template files one kind of finding, so it stamps that kind into
/// the `task.with(…)` head. `task.typ` files anything, so it leaves `kind` out
/// and `mt add` splices a choice over the configured vocabulary instead of a
/// second copy of it. `tweak.typ` files anything too, done, where the commit
/// subject carries the kind. Matched at the head's own two-space indent; a body
/// `slot(…, kind: "text")` argument sits there too, but never in the head,
/// which ends at the first `)` line.
#[test]
fn dogfood_templates_stamp_a_known_kind_or_leave_it_to_config() {
    let config = config();
    let allowed = &config.tasks.fields.get("kind").unwrap().values;
    let dir = root().join(".mindtape/templates");
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let source = std::fs::read_to_string(&path).unwrap();
        let kind = source
            .lines()
            .take_while(|line| *line != ")")
            .find_map(|line| line.strip_prefix("  kind: \""))
            .and_then(|rest| rest.strip_suffix("\","));
        let generic =
            ["task.typ", "tweak.typ"].contains(&path.file_name().unwrap().to_str().unwrap());
        match kind {
            None => assert!(generic, "{} stamps no kind", mindtape::display_path(path)),
            Some(kind) => assert!(
                !generic && allowed.iter().any(|value| value == kind),
                "{} stamps `{kind}`; allowed {allowed:?}",
                mindtape::display_path(path)
            ),
        }
    }
}
