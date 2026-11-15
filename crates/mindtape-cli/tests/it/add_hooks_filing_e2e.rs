//! The filing policy this repo declares for `[[on.add]]`: the month hook and
//! the sweep hook, each choosing its own folder from the draft alone.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use super::add_hooks_support::{add_pinned, project, read};

const POLICY: &str = r#"
[[on.add]]
name = "file-by-month"
when = '!has_tag("swept")'
actions = [{ dir = "tasks/{date:YYYY}-{date:MM}" }]

[[on.add]]
name = "sweeps"
when = 'has_tag("swept")'
actions = [{ dir = "tasks/swept" }]
"#;

/// A pinned add that must land, with the stderr as the assertion message.
fn filed(dir: &Path, args: &[&str]) {
    let (_, stderr, ok) = add_pinned(dir, "2026-11-13", args);
    assert!(ok, "{stderr}");
}

/// A filed task follows the pinned month, and a task carrying `swept` goes to
/// the one inbox folder instead. The hook reads the tag on the draft. A sweep
/// passes only `--set tags+=swept` and specifies no path.
#[test]
fn the_month_hook_and_the_swept_hook_choose_their_own_folders() {
    let dir = project(POLICY);
    filed(dir.path(), &["add", "--name", "monthly", "Monthly"]);
    assert!(
        read(dir.path(), "tasks/2026-11/monthly.typ").contains("title: \"Monthly\""),
        "a filed task lands in its month folder"
    );

    filed(
        dir.path(),
        &[
            "add",
            "--name",
            "batched",
            "--set",
            "tags+=swept",
            "Batched",
        ],
    );
    let written = read(dir.path(), "tasks/swept/batched.typ");
    assert!(written.contains("title: \"Batched\""), "{written}");
    assert!(written.contains("\"swept\""), "{written}");
}
