//! Tests for the editor loop: a buffer the fill refuses re-opens with the
//! error above the values, and a buffer a rule refuses re-opens with every
//! breach. The answers survive a typo and a rejection.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use mindtape::test_support::tempdir;
use mindtape::{AddConfig, RawValue, SlotSpec, discover_slots, parse_config};

use super::{
    Ready, Session, field_aliases, fill_until_it_passes,
    interactive::{Fill, fill_until_it_fills},
};
use crate::cmd::add::AddArgs;
use crate::cmd::add::interactive::Edited;
use crate::cmd::add::values::{Collected, collect_values};
use crate::pre_write::Problem;
use crate::test_util::make_project;
use crate::util::Project;

const TEMPLATE: &str = mindtape::typst_task!(
    r#"(
  title: slot("title"),
  priority: slot("priority", kind: "int", default: 5),
)"#
);

/// A template whose `AS-IS` body is a raw slot, the shape the `fix` template
/// uses for the defect step.
const RULE_TEMPLATE: &str = mindtape::typst_task!(
    r#"(
  title: slot("title"),
)

== AS-IS

#slot("as-is", kind: "raw")
"#
);

const BAD_BUFFER: &str = "\
title: \"Sort tasks without a due date last\"
as-is: |-
  + Run `mt ls --sort due:asc`.
  + The undated tasks print first.
";

const GOOD_BUFFER: &str = "\
title: \"Sort tasks without a due date last\"
as-is: |-
  + Run `mt ls --sort due:asc`.
  + 🔴 The undated tasks print first.
";

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

/// A buffer whose `AS-IS` has no 🔴 comes back with the breach above the
/// values. The loop lands the task once the buffer carries the step.
#[test]
fn a_rule_breach_re_opens_the_buffer_with_every_breach() {
    let dir = tempdir();
    let seen = dir.path().join("seen.yaml");
    let bad = dir.path().join("bad.yaml");
    let good = dir.path().join("good.yaml");
    std::fs::write(&bad, BAD_BUFFER).unwrap();
    std::fs::write(&good, GOOD_BUFFER).unwrap();
    let editor = retrying_editor(dir.path(), &seen, &bad, &good);
    let fixture = Fixture::new();
    let mut session = fixture.session();
    let check = |ready: &Ready| {
        if ready.filled.contains('🔴') {
            return Ok(None);
        }
        Ok(Some(vec![Problem {
            line: "<new task from template 'task'>:33: \
                   body.content.rules.mark-defect-step-with-red: \
                   open the step that shows the defect with 🔴"
                .to_owned(),
            detail: Some("section `AS-IS` must match `(?m)^\\s*(\\+|\\d+\\.)\\s+🔴`".to_owned()),
        }]))
    };

    let ready =
        fill_until_it_passes(&mut session, &AddConfig::default(), Some(&editor), &check).unwrap();

    assert!(ready.filled.contains('🔴'), "{}", ready.filled);
    let second = std::fs::read_to_string(&seen).unwrap();
    assert!(second.starts_with("# error: "), "{second}");
    assert!(
        second.contains("# note: section `AS-IS` must match"),
        "{second}"
    );
    assert!(
        second.contains("+ Run `mt ls --sort due:asc`."),
        "the answers stay: {second}"
    );
}

/// An editor that saves the same refused buffer every round cannot fix the
/// breach by re-opening it. The loop stops after the second identical save.
/// The final check parks the task instead of spinning.
#[test]
fn an_unchanged_buffer_stops_the_loop() {
    let dir = tempdir();
    let count = dir.path().join("count");
    let bad = dir.path().join("bad.yaml");
    std::fs::write(&bad, BAD_BUFFER).unwrap();
    let editor = counting_editor(dir.path(), &count, &bad);

    let (calls, ready) = run_refused(&editor, &count);

    assert_eq!(calls, "2");
    assert_eq!(
        ready.values["title"],
        RawValue::Str("Sort tasks without a due date last".to_owned())
    );
}

/// `--no-verify` skips the loop's check, as it skips the final one.
#[test]
fn no_verify_skips_the_candidate_check() {
    let fixture = Fixture::new();
    let args = AddArgs {
        no_verify: true,
        ..AddArgs::default()
    };
    let ready = Ready {
        filled: String::new(),
        values: BTreeMap::new(),
        dir: None,
        explicit: BTreeSet::new(),
        env_keys: BTreeSet::new(),
        touched: BTreeSet::new(),
        interactive: true,
        verify: false,
    };

    let problems = super::check_candidate(&args, &fixture.project, "task", &ready).unwrap();

    assert!(problems.is_none());
}

/// Run the loop on `editor` with a check that always refuses, and return the
/// editor's invocation count and the values the loop stopped on.
fn run_refused(editor: &str, count: &Path) -> (String, Ready) {
    let fixture = Fixture::new();
    let mut session = fixture.session();
    let check = |_ready: &Ready| {
        Ok(Some(vec![Problem {
            line: "task:1: rules.one: refused".to_owned(),
            detail: None,
        }]))
    };
    let ready =
        fill_until_it_passes(&mut session, &AddConfig::default(), Some(editor), &check).unwrap();
    let calls = std::fs::read_to_string(count).unwrap().trim().to_owned();
    (calls, ready)
}

/// The project and values a [`RULE_TEMPLATE`] session renders, kept alive for
/// the session's borrows.
struct Fixture {
    _root: tempfile::TempDir,
    args: AddArgs,
    project: Project,
    specs: Vec<SlotSpec>,
    collected: Collected,
    aliases: BTreeMap<String, String>,
    empty: BTreeMap<String, RawValue>,
}

impl Fixture {
    fn new() -> Self {
        let (root, project) = make_project(&[], parse_config("").unwrap());
        Self {
            _root: root,
            args: AddArgs::default(),
            project,
            specs: discover_slots(RULE_TEMPLATE),
            collected: collect_values(&[], &[], &[]).unwrap(),
            aliases: field_aliases(RULE_TEMPLATE),
            empty: BTreeMap::new(),
        }
    }

    /// A session over [`RULE_TEMPLATE`], with no caller values and no fields
    /// set.
    fn session(&self) -> Session<'_> {
        Session {
            args: &self.args,
            project: &self.project,
            template: "task",
            source: RULE_TEMPLATE,
            specs: &self.specs,
            collected: &self.collected,
            aliases: &self.aliases,
            seed: &self.empty,
            explicit: BTreeSet::new(),
            env_keys: BTreeSet::new(),
            status: None,
            verify: true,
        }
    }
}

/// Run the fill loop on an editor that writes `bad` first, then the fixed
/// buffer once the error line is above the values. Returns the values the loop
/// lands on and the buffer the second session opened.
fn fill_after(bad: &str) -> (Edited, String) {
    let dir = tempdir();
    let seen = dir.path().join("seen.yaml");
    let bad_path = dir.path().join("bad.yaml");
    let good_path = dir.path().join("good.yaml");
    std::fs::write(&bad_path, bad).unwrap();
    std::fs::write(&good_path, "title: \"Sort tasks\"\npriority: 5\n").unwrap();
    let editor = retrying_editor(dir.path(), &seen, &bad_path, &good_path);
    let specs = discover_slots(TEMPLATE);

    let fill = Fill {
        specs: &specs,
        source: TEMPLATE,
        seed: &BTreeMap::new(),
        verify: true,
    };
    let edited = fill_until_it_fills(&AddConfig::default(), Some(&editor), &fill, None).unwrap();
    (edited, std::fs::read_to_string(&seen).unwrap())
}

/// An editor command that copies the buffer it receives to `seen`, then writes
/// `good` once an error line is above the values, else `bad`. One function with
/// two `#[cfg]` bodies, for the reason `fake_editor` gives.
fn retrying_editor(dir: &Path, seen: &Path, bad: &Path, good: &Path) -> String {
    #[cfg(not(windows))]
    {
        let script = dir.join("retrying-editor.sh");
        std::fs::write(
            &script,
            format!(
                "cp \"$1\" \"{seen}\"\n\
                 if grep -q '^# error: ' \"$1\"; then\n\
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
    #[cfg(windows)]
    {
        let script = dir.join("retrying-editor.cmd");
        std::fs::write(
            &script,
            format!(
                "@copy /Y \"%~1\" \"{seen}\" >nul\r\n\
                 @findstr /B /C:\"# error: \" \"%~1\" >nul\r\n\
                 @if errorlevel 1 goto bad\r\n\
                 @copy /Y \"{good}\" \"%~1\" >nul\r\n\
                 @goto :eof\r\n\
                 :bad\r\n\
                 @copy /Y \"{bad}\" \"%~1\" >nul\r\n",
                seen = seen.display(),
                bad = bad.display(),
                good = good.display()
            ),
        )
        .unwrap();
        format!("cmd /C {}", script.display())
    }
}

/// An editor that saves `bad` every round and counts its invocations.
fn counting_editor(dir: &Path, count: &Path, bad: &Path) -> String {
    #[cfg(not(windows))]
    {
        let script = dir.join("counting-editor.sh");
        std::fs::write(
            &script,
            format!(
                "n=$(cat \"{count}\" 2>/dev/null || echo 0)\n\
                 echo $((n + 1)) > \"{count}\"\n\
                 cp \"{bad}\" \"$1\"\n",
                count = count.display(),
                bad = bad.display()
            ),
        )
        .unwrap();
        format!("sh {}", script.display())
    }
    #[cfg(windows)]
    {
        let script = dir.join("counting-editor.cmd");
        std::fs::write(
            &script,
            format!(
                "@set /a n=0\r\n\
                 @if exist \"{count}\" set /p n=<\"{count}\"\r\n\
                 @set /a n=n+1\r\n\
                 @echo %n%>\"{count}\"\r\n\
                 @copy /Y \"{bad}\" \"%~1\" >nul\r\n",
                count = count.display(),
                bad = bad.display()
            ),
        )
        .unwrap();
        format!("cmd /C {}", script.display())
    }
}

/// An editor that keeps changing the buffer without fixing the breach cannot
/// be re-opened forever. The round bound stops the loop, and the final check
/// parks the task.
#[cfg(unix)]
#[test]
fn the_round_bound_stops_a_changing_buffer() {
    let dir = tempdir();
    let count = dir.path().join("count");
    let editor = changing_editor(dir.path(), &count);

    let (calls, ready) = run_refused(&editor, &count);

    assert_eq!(calls, "20");
    assert_eq!(
        ready.values["title"],
        RawValue::Str("Sort tasks".to_owned())
    );
}

/// An editor that appends a round comment every time. The buffer changes
/// without fixing the breach, and the count records its invocations.
#[cfg(unix)]
fn changing_editor(dir: &Path, count: &Path) -> String {
    let script = dir.join("changing-editor.sh");
    std::fs::write(
        &script,
        format!(
            "n=$(cat \"{count}\" 2>/dev/null || echo 0)\n\
             echo $((n + 1)) > \"{count}\"\n\
             printf 'title: \"Sort tasks\"\\nas-is: |-\\n  + Run `mt ls`.\\n# round %s\\n' \"$n\" > \"$1\"\n",
            count = count.display()
        ),
    )
    .unwrap();
    format!("sh {}", script.display())
}

/// `--verify` and `--no-verify` win over `[add] verify-when`, and an absent
/// key verifies always. The test process has no terminal, so `tty()` is false
/// and `!tty()` is true.
#[test]
fn verify_reads_the_flags_then_the_config() {
    let cases = [
        (None, false, false, true),
        (None, true, false, true),
        (None, false, true, false),
        (Some("false"), false, false, false),
        (Some("false"), true, false, true),
        (Some("!tty()"), false, false, true),
        (Some("tty()"), false, false, false),
    ];
    for (expr, verify, no_verify, want) in cases {
        let config = match expr {
            Some(expr) => format!("[add]\nverify-when = \"{expr}\"\n"),
            None => String::new(),
        };
        let (_root, project) = make_project(&[], parse_config(&config).unwrap());
        let args = AddArgs {
            verify,
            no_verify,
            ..AddArgs::default()
        };

        assert_eq!(
            super::verify(&args, &project).unwrap(),
            want,
            "verify-when = {expr:?}, --verify = {verify}, --no-verify = {no_verify}"
        );
    }
}
