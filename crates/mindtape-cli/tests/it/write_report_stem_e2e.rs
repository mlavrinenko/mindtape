//! End-to-end: every write command names a task by its stem in its report,
//! whatever the user typed for it: the stem, a path or a stem hash.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use mindtape::StemHashes;

use crate::support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n";

const ALPHA: &str = mindtape::typst_task!(
    "(\n  title: \"Task alpha\",\n  status: proposed(2026, 8, 1),\n  priority: 5,\n)\n"
);
const BRAVO: &str = mindtape::typst_task!(
    "(\n  title: \"Task bravo\",\n  status: proposed(2026, 8, 1),\n  priority: 5,\n)\n"
);
const STANDUP: &str = mindtape::typst_task!(
    "(\n  title: \"Standup\",\n  status: wip(2026, 6, 1),\n  priority: 3,\n  \
     recur: every(weeks: 1, materialize: \"mutate\"),\n)\n"
);

/// A project holding `alpha`, `bravo` and the recurring `standup`.
fn project() -> tempfile::TempDir {
    support::scaffold_config(
        CONFIG,
        &[
            ("alpha.typ", ALPHA),
            ("bravo.typ", BRAVO),
            ("standup.typ", STANDUP),
        ],
    )
}

/// The stem hash `mt ls` prints for `stem` in [`project`].
fn hash(stem: &str) -> String {
    let stems = ["alpha", "bravo", "standup"];
    StemHashes::new(stems).hash_of(stem).unwrap().to_owned()
}

fn first_line(dir: &Path, args: &[&str]) -> String {
    let out = support::stdout(dir, args);
    out.lines().next().unwrap_or_default().to_owned()
}

#[test]
fn a_flip_names_the_stem_for_a_hash_or_a_path() {
    let dir = project();
    let alpha = hash("alpha");

    assert_eq!(
        first_line(dir.path(), &["wip", &alpha]),
        "flipped alpha (Task alpha): proposed → wip"
    );
    assert_eq!(
        first_line(dir.path(), &["wip", "tasks/alpha.typ"]),
        "re-stamped alpha (Task alpha): wip (today)"
    );
}

#[test]
fn set_and_link_name_the_stem_for_a_hash() {
    let dir = project();
    let alpha = hash("alpha");

    assert_eq!(
        first_line(dir.path(), &["set", &alpha, "assignee=ada"]),
        "set alpha (Task alpha): assignee: \"ada\""
    );
    let linked = first_line(dir.path(), &["link", &alpha, "depends-on", "bravo"]);
    assert!(linked.starts_with("linked alpha: "), "{linked}");
}

#[test]
fn review_names_the_stem_for_a_hash() {
    let dir = project();
    let alpha = hash("alpha");

    let reviewed = first_line(dir.path(), &["review", &alpha]);
    assert!(
        reviewed.starts_with("reviewed alpha (Task alpha): "),
        "{reviewed}"
    );
    assert_eq!(
        first_line(dir.path(), &["review", &alpha]),
        "unchanged: alpha was already reviewed today"
    );
}

#[test]
fn recur_names_the_stem_for_a_hash() {
    let dir = project();
    let standup = hash("standup");

    let skipped = first_line(dir.path(), &["recur", &standup, "--skip"]);
    assert!(
        skipped.starts_with("skipped standup (Standup): now due "),
        "{skipped}"
    );
    let refreshed = first_line(dir.path(), &["recur", &standup]);
    assert!(
        refreshed.starts_with("refreshed standup (Standup): due "),
        "{refreshed}"
    );
}

#[test]
fn closing_a_recurring_task_names_its_stem_for_a_hash() {
    let dir = project();
    let out = support::stdout(dir.path(), &["done", &hash("standup")]);

    assert!(
        out.contains("standup is recurring (mutate), so closing it ends the series"),
        "{out}"
    );
}

/// A spawn introduces a new file, so it names the file as `mt add` does: by its
/// path from the project root.
#[test]
fn a_spawn_names_the_new_file_from_the_project_root() {
    let spawn = STANDUP.replace(", materialize: \"mutate\"", "");
    let dir = support::scaffold_config(CONFIG, &[("standup.typ", &spawn)]);

    let spawned = first_line(dir.path(), &["recur", "standup"]);
    assert!(spawned.starts_with("spawned tasks/standup-"), "{spawned}");
}
