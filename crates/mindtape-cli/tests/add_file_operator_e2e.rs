//! End-to-end tests for the `key@path` operator: `mt add -- summary@notes.txt`
//!
//! Its whole design is that the SLOT'S KIND decides what the file's bytes
//! become, and that nothing between the file and the slot converts anything.
//! That is only provable through the binary — the parser hands every operator
//! the same `RawValue::Str`, so a unit test cannot tell a `text` fill from a
//! `raw` one. Each test here reads the written task back.
//!
//! Its own binary rather than more of `add_e2e.rs`, along the same subject
//! seam `add_config_scaffold_e2e.rs` follows.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

mod support;

use std::path::Path;

/// Write `body` to `<dir>/<name>` and return the path as a token would spell
/// it. Absolute, because `mt` runs with `dir` as its cwd but the token is
/// resolved by the process, not by the project root.
fn fixture(dir: &Path, name: &str, body: &str) -> String {
    let path = dir.join(name);
    std::fs::write(&path, body).unwrap();
    path.to_string_lossy().into_owned()
}

/// A `text` slot markup-escapes the file's bytes exactly as it escapes a
/// `--set` value. The file is a carrier and nothing more: prose that happens
/// to contain markup leaders is still prose, and it reads back as typed.
///
/// This is why the docs spell the example `.txt` and never `.md` — nothing
/// converts Markdown to Typst, so a `#` in the file is a literal `#`, not a
/// heading.
#[test]
fn a_text_slot_escapes_the_files_bytes_like_any_other_value() {
    let dir = support::project_with_slot_template();
    let hazards = support::MARKUP_HAZARDS;
    let path = fixture(dir.path(), "notes.txt", &format!("{hazards}\n"));

    support::mt(dir.path())
        .args(["add", "--name", "from-file", "--set", "title=From file"])
        .args(["--set", "why=a note"])
        .args(["--", &format!("summary@{path}")])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/from-file.typ")).unwrap();
    assert!(
        written.contains(r"create-\*star\* \_u\_ \$x\$ \#h \[b\] \<a> >g \@at"),
        "a text slot must escape a file's bytes: {written}"
    );
    support::mt(dir.path()).arg("check").assert().success();
}

/// A `raw` slot splices the file verbatim — the case the operator exists for.
/// Write the `.typ`, check it, then file the task; the markup never has to
/// survive a shell argument.
#[test]
fn a_raw_slot_splices_the_file_verbatim() {
    let dir = support::project_with_raw_slot_template();
    let body = "- Carve the file along a seam that means something.\n\
                - Or raise its cap in `.linecop.yaml`, with a reason.\n";
    let path = fixture(dir.path(), "remedy.typ", body);

    support::mt(dir.path())
        .args(["add", "--name", "from-typ", "--set", "title=From typ"])
        .args(["--", &format!("summary@{path}")])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/from-typ.typ")).unwrap();
    assert!(
        written.contains("- Or raise its cap in `.linecop.yaml`, with a reason."),
        "a raw slot must splice the file verbatim: {written}"
    );
    assert!(
        !written.contains(r"\`"),
        "a raw slot must not escape the file: {written}"
    );
    support::mt(dir.path()).arg("check").assert().success();
}

/// The operator reads the file for EVERY kind, not only the two the design
/// argues for. It is a carrier, not a converter: gating it by kind would
/// invent a second diagnostic for a mistake the fill layer already reports by
/// name, and would buy nothing a caller can act on.
///
/// The trailing newline every editor writes is what makes this true in
/// practice rather than only on paper — an `int` slot fed `7\n` fills as `7`.
#[test]
fn a_typed_slot_reads_the_file_too() {
    let dir = support::project_with_slot_template();
    let path = fixture(dir.path(), "weight.txt", "7\n");

    support::mt(dir.path())
        .args(["add", "--name", "typed", "--set", "title=Typed"])
        .args(["--set", "why=a note"])
        .args(["--", &format!("weight@{path}")])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/typed.typ")).unwrap();
    assert!(written.contains("priority: 7,"), "{written}");
    assert_eq!(
        support::at(&support::task_json(dir.path(), "typed"), "/priority").as_f64(),
        Some(7.0)
    );
}

/// A file whose contents do not fit the slot fails at the same place a
/// `--set` value does, naming the slot — the reason no kind gate is needed
/// upstream of it.
#[test]
fn a_file_that_does_not_fit_the_kind_fails_naming_the_slot() {
    let dir = support::project_with_slot_template();
    let path = fixture(dir.path(), "weight.txt", "heavy\n");

    let output = support::mt(dir.path())
        .args(["add", "--name", "bad-typed", "--set", "title=Bad"])
        .args(["--set", "why=a note"])
        .args(["--", &format!("weight@{path}")])
        .assert()
        .failure()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("'weight'"), "{stderr}");
    assert!(
        !dir.path().join("tasks/bad-typed.typ").exists(),
        "nothing may be written when a slot rejects its value"
    );
}

/// Both carriers, one grammar: `--set` takes `@` exactly as the bare tokens
/// do. Asserting only the `--` form would let the flag form drift.
#[test]
fn the_set_flag_carries_the_file_operator_too() {
    let dir = support::project_with_slot_template();
    let path = fixture(dir.path(), "why.txt", "because the file says so");

    support::mt(dir.path())
        .args(["add", "--name", "via-set", "--set", "title=Via set"])
        .args(["--set", &format!("why@{path}")])
        .assert()
        .success();

    assert_eq!(
        support::at(&support::task_json(dir.path(), "via-set"), "/links/0/note").as_str(),
        Some("because the file says so")
    );
}

/// An unreadable file must fail before anything is written, naming the slot
/// as well as the path — one `mt add` can carry many tokens.
#[test]
fn an_unreadable_file_fails_before_writing() {
    let dir = support::project_with_slot_template();

    let output = support::mt(dir.path())
        .args(["add", "--name", "missing", "--set", "title=Missing"])
        .args(["--", "summary@no/such/notes.txt"])
        .assert()
        .failure()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: can't read slot file: 'no/such/notes.txt' for slot 'summary'"),
        "{stderr}"
    );
    assert!(
        !dir.path().join("tasks/missing.typ").exists(),
        "a bad token may not leave a file behind"
    );
}
