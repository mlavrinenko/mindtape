//! The canonical task-source contract in `www/content/concepts.typ` is the one
//! statement of which files count as tasks, so every page that names a task
//! file links to it. `tola validate` checks routes, not anchors, and the docs
//! gates read prose and spans rather than links, so this holds the set.
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

/// The anchor of the `== Which files count as tasks` section.
const ANCHOR: &str = "/concepts/#which-files-count-as-tasks";

/// Every page that names which files are tasks.
const PAGES: [&str; 12] = [
    "www/content/concepts.typ",
    "www/content/templates.typ",
    "www/content/config.typ",
    "www/content/task-files.typ",
    "www/content/commit-rules.typ",
    "www/content/trace-commits-to-tasks.typ",
    "www/content/cli/add.typ",
    "www/content/cli/import.typ",
    "www/content/cli/recur.typ",
    "www/content/cli/ls.typ",
    "www/content/cli/check.typ",
    "www/content/cli/fix.typ",
];

fn root() -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "..", ".."].iter().collect()
}

#[test]
fn docs_task_source_pages_link_the_contract() {
    let root = root();
    let mut failures = Vec::new();
    for page in PAGES {
        let path = root.join(page);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{}: {error}", mindtape::display_path(&path)));
        if !source.contains(ANCHOR) {
            failures.push(page);
        }
    }
    assert!(
        failures.is_empty(),
        "these pages do not link `{ANCHOR}`:\n{}",
        failures.join("\n")
    );

    let concepts = std::fs::read_to_string(root.join("www/content/concepts.typ")).unwrap();
    assert!(
        concepts.contains("== Which files count as tasks"),
        "concepts.typ has no `== Which files count as tasks` heading"
    );
}
