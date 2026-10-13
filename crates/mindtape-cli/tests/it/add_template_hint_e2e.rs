//! A refused `mt add` points at the template that pins what the caller passed:
//! the default template left `kind` to a spliced slot, another template writes
//! `kind: "dev"` literally, and the rule the task breaks is why the first
//! cannot file it.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;
use mindtape::test_support::write_file;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.fields.kind]\nvalues = [\"feat\", \"dev\"]\nrequired = true\n\
     [[tasks.rules]]\nname = \"dev-sets-difficulty\"\n\
     when = 'kind == \"dev\"'\nassert = 'kind != \"dev\" || difficulty == 3'\n\
     message = \"a dev chore carries difficulty 3\"\nseverity = \"error\"\n";

const PINNED: &str =
    mindtape::typst_task!("(\n  title: slot(\"title\"),\n  kind: \"dev\",\n  difficulty: 3,\n)\n");

fn scaffold() -> tempfile::TempDir {
    let dir = support::scaffold_project(CONFIG);
    std::fs::create_dir_all(dir.path().join(".mindtape/templates")).unwrap();
    write_file(dir.path(), ".mindtape/templates/dev.typ", PINNED);
    dir
}

fn add(dir: &std::path::Path, kind: &str, extra: &[&str]) -> assert_cmd::assert::Assert {
    support::mt(dir)
        .args(["add", "--name", "t", "--set", "title=T"])
        .args(["--set", kind])
        .args(extra)
        .assert()
}

#[test]
fn refusal_names_the_template_that_pins_the_value() {
    let dir = scaffold();
    let out = add(dir.path(), "kind=dev", &[]).failure();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        stderr
            .contains("hint: file it with the template that sets that field itself: mt add -t dev"),
        "{stderr}"
    );
}

#[test]
fn no_hint_when_no_template_pins_the_value() {
    let dir = scaffold();
    let out = add(dir.path(), "kind=feat", &["--set", "kind=bogus"]).failure();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(!stderr.contains("hint:"), "{stderr}");
}

/// A breach of a rule that reads only `title` is not fixed by a template that
/// pins `kind`, so the refusal names no template.
#[test]
fn no_hint_when_the_breach_names_another_field() {
    let dir = scaffold();
    let config = format!(
        "{CONFIG}[[tasks.rules]]\nname = \"no-bare-t\"\nassert = 'title != \"T\"'\n\
         severity = \"error\"\n"
    );
    write_file(dir.path(), ".mindtape/config.toml", &config);
    write_file(
        dir.path(),
        ".mindtape/templates/feat.typ",
        PINNED.replace("\"dev\"", "\"feat\""),
    );
    let out = add(dir.path(), "kind=feat", &[]).failure();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("rules.no-bare-t"), "{stderr}");
    assert!(!stderr.contains("hint:"), "{stderr}");
}

/// A body header rule is about the names its `when` reads, not about `body`:
/// the rule below breaches because the filed kind is `dev`, so the `dev`
/// template that writes the Rollback section is the fix.
#[test]
fn a_body_header_rule_is_about_the_field_its_when_reads() {
    let dir = scaffold();
    // Only the body rule can refuse here, so the hint cannot come from another
    // breach's field.
    let config = "[tasks]\nroots = [\"tasks\"]\n\
         [tasks.fields.kind]\nvalues = [\"feat\", \"dev\"]\nrequired = true\n\
         [[tasks.body.headers.rules]]\nname = \"rollback-for-dev\"\n\
         path = [\"Rollback\"]\naction = \"require\"\nwhen = 'kind == \"dev\"'\n\
         message = \"a dev chore says how to undo it\"\n";
    write_file(dir.path(), ".mindtape/config.toml", config);
    write_file(
        dir.path(),
        ".mindtape/templates/dev.typ",
        format!("{PINNED}\n== Rollback\n\n- Revert it.\n"),
    );
    let out = add(dir.path(), "kind=dev", &[]).failure();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("rollback-for-dev"), "{stderr}");
    assert!(
        stderr
            .contains("hint: file it with the template that sets that field itself: mt add -t dev"),
        "{stderr}"
    );
}

/// The caller passes the slot's name, not the field's: `kind: slot("type")`
/// is filled by `--set type=dev`, and the hint still names the field.
#[test]
fn a_field_filled_by_a_differently_named_slot_still_hints() {
    let dir = scaffold();
    write_file(
        dir.path(),
        ".mindtape/templates/typed.typ",
        mindtape::typst_task!("(\n  title: slot(\"title\"),\n  kind: slot(\"type\"),\n)\n"),
    );
    let out = support::mt(dir.path())
        .args(["add", "-t", "typed", "--name", "t", "--set", "title=T"])
        .args(["--set", "type=dev"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("rules.dev-sets-difficulty"), "{stderr}");
    assert!(
        stderr
            .contains("hint: file it with the template that sets that field itself: mt add -t dev"),
        "{stderr}"
    );
}
