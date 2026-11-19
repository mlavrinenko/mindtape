//! `mt add --edit` re-opens the buffer when a rule refuses the filled task.
//!
//! Spawns the binary under `script`, which provides a pseudo-terminal, so the
//! `--edit` tty check passes. `$EDITOR` points at a script that saves a buffer
//! without the 🔴 step first, then the fixed one once the breach is above the
//! values. The task lands, and the second buffer carried the breach.
//!
//! Unix only, for the same reason as `tty_e2e.rs`: the pty comes from
//! util-linux `script`, which Windows has no equivalent of.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.body.content.rules]]\n\
     name = \"mark-defect-step-with-red\"\n\
     path = [\"AS-IS\"]\n\
     assert = { must-match = \"(?m)^\\\\s*(\\\\+|\\\\d+\\\\.)\\\\s+🔴\" }\n\
     message = \"open the step that shows the defect with 🔴\"\n";

const TEMPLATE: &str = mindtape::typst_task!(
    r#"(
  title: slot("title"),
)

== AS-IS

#slot("as-is", kind: "raw")
"#
);

const BAD: &str = "\
title: \"Sort tasks\"
as-is: |-
  + Run `mt ls`.
  + The undated tasks print first.
";

const GOOD: &str = "\
title: \"Sort tasks\"
as-is: |-
  + Run `mt ls`.
  + 🔴 The undated tasks print first.
";

#[test]
fn a_rule_breach_re_opens_the_buffer_and_files_the_fixed_task() {
    let dir = support::scaffold_project(CONFIG);
    let templates = dir.path().join(".mindtape/templates");
    std::fs::create_dir_all(&templates).unwrap();
    std::fs::write(templates.join("task.typ"), TEMPLATE).unwrap();
    let seen = dir.path().join("seen.yaml");
    let bad = dir.path().join("bad.yaml");
    let good = dir.path().join("good.yaml");
    std::fs::write(&bad, BAD).unwrap();
    std::fs::write(&good, GOOD).unwrap();
    let editor = editor_script(dir.path(), &seen, &bad, &good);

    let mt = env!("CARGO_BIN_EXE_mt");
    let mut script = std::process::Command::new("script");
    let output = support::isolate(&mut script, dir.path())
        .args([
            "-q",
            "-e",
            "-c",
            &format!("{mt} add --edit -t task"),
            "/dev/null",
        ])
        .current_dir(dir.path())
        .env("EDITOR", &editor)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let written = std::fs::read_to_string(dir.path().join("tasks/sort-tasks.typ")).unwrap();
    assert!(written.contains('🔴'), "{written}");
    let second = std::fs::read_to_string(&seen).unwrap();
    assert!(second.starts_with("# error: "), "{second}");
    assert!(
        second.contains("# note: section `AS-IS` must match"),
        "{second}"
    );
    assert!(
        second.contains("+ Run `mt ls`."),
        "the answers stay: {second}"
    );
}

/// An editor that copies the buffer it receives to `seen`, then writes `good`
/// once an error line is above the values, else `bad`.
fn editor_script(dir: &Path, seen: &Path, bad: &Path, good: &Path) -> String {
    let script = dir.join("editor.sh");
    std::fs::write(
        &script,
        format!(
            "cp \"$1\" \"{seen}\"\n\
             if head -1 \"$1\" | grep -q '^# error: '; then\n\
             \x20 cp \"{good}\" \"$1\"\n\
             else\n\
             \x20 cp \"{bad}\" \"$1\"\n\
             fi\n",
            seen = seen.display(),
            bad = bad.display(),
            good = good.display()
        ),
    )
    .unwrap();
    format!("sh {}", script.display())
}
