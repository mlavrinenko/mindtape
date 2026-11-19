//! Tests for the editor loop: a buffer the fill refuses re-opens with the
//! error above the values. The answers survive a typo.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use mindtape::test_support::tempdir;
use mindtape::{AddConfig, RawValue, discover_slots};

use super::fill_until_it_fills;
use crate::cmd::add::interactive::Edited;

const TEMPLATE: &str = mindtape::typst_task!(
    r#"(
  title: slot("title"),
  priority: slot("priority", kind: "int", default: 5),
)"#
);

#[test]
fn a_refused_buffer_re_opens_with_the_error_above_the_values() {
    for (bad, error) in [
        (
            "title: \"Sort tasks\"\npriority: five\n",
            "# error: slot has the wrong type: 'priority': expects int, got string\n",
        ),
        (
            "- not a mapping\n",
            "# error: values must be a YAML mapping\n",
        ),
    ] {
        let (edited, second) = fill_after(bad);

        assert_eq!(edited.values["priority"], RawValue::Int(5));
        assert!(second.starts_with(error), "{second}");
        assert!(second.contains(bad.trim_end()), "{second}");
    }
}

/// Run the loop on an editor that writes `bad` first, then the fixed buffer
/// once the error line is above the values. Returns the values the loop lands
/// on and the buffer the second session opened.
fn fill_after(bad: &str) -> (Edited, String) {
    let dir = tempdir();
    let seen = dir.path().join("seen.yaml");
    let bad_path = dir.path().join("bad.yaml");
    std::fs::write(&bad_path, bad).unwrap();
    let editor = retrying_editor(dir.path(), &seen, &bad_path);
    let specs = discover_slots(TEMPLATE);

    let edited = fill_until_it_fills(
        &AddConfig::default(),
        Some(&editor),
        &specs,
        TEMPLATE,
        &BTreeMap::new(),
    )
    .unwrap();
    (edited, std::fs::read_to_string(&seen).unwrap())
}

/// An editor command that copies the buffer it receives to `seen`, then writes
/// the fixed buffer once an error line is above the values, else `bad`. One
/// function with two `#[cfg]` bodies, for the reason `fake_editor` gives.
fn retrying_editor(dir: &std::path::Path, seen: &std::path::Path, bad: &std::path::Path) -> String {
    #[cfg(not(windows))]
    {
        let script = dir.join("retrying-editor.sh");
        std::fs::write(
            &script,
            format!(
                "cp \"$1\" \"{seen}\"\n\
                 if grep -q '^# error: ' \"$1\"; then\n\
                 \x20 printf 'title: \"Sort tasks\"\\npriority: 5\\n' > \"$1\"\n\
                 else\n\
                 \x20 cp \"{bad}\" \"$1\"\n\
                 fi\n",
                seen = seen.display(),
                bad = bad.display()
            ),
        )
        .unwrap();
        format!("sh {}", script.display())
    }
    #[cfg(windows)]
    {
        let script = dir.join("retrying-editor.cmd");
        std::fs::write(
            &script,
            format!(
                "@copy /Y \"%~1\" \"{seen}\" >nul\r\n\
                 @findstr /B /C:\"# error: \" \"%~1\" >nul\r\n\
                 @if errorlevel 1 goto bad\r\n\
                 (echo title: \"Sort tasks\")>\"%~1\"\r\n\
                 (echo priority: 5)>>\"%~1\"\r\n\
                 @goto :eof\r\n\
                 :bad\r\n\
                 @copy /Y \"{bad}\" \"%~1\" >nul\r\n",
                seen = seen.display(),
                bad = bad.display()
            ),
        )
        .unwrap();
        format!("cmd /C {}", script.display())
    }
}
