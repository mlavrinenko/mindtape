//! A refused `mt add` points at the template that pins what the caller passed:
//! the default template left `kind` to a spliced slot, another template writes
//! `kind: "dev"` literally, and the rule the task breaks is why the first
//! cannot file it.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

mod support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.fields.kind]\nvalues = [\"feat\", \"dev\"]\nrequired = true\n\
     [[tasks.rules]]\nname = \"dev-sets-difficulty\"\n\
     when = 'kind == \"dev\"'\nassert = 'kind != \"dev\" || difficulty == 3'\n\
     message = \"a dev chore carries difficulty 3\"\nseverity = \"error\"\n";

const PINNED: &str = "#import \"@local/mindtape:0.2.0\": *\n\
     #show: task.with(\n  title: slot(\"title\"),\n  kind: \"dev\",\n  difficulty: 3,\n)\n";

fn scaffold(dir: &std::path::Path) {
    std::fs::create_dir_all(dir.join(".mindtape/templates")).unwrap();
    std::fs::write(dir.join(".mindtape/config.toml"), CONFIG).unwrap();
    std::fs::write(dir.join(".mindtape/templates/dev.typ"), PINNED).unwrap();
    support::install_lib(dir);
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
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    let out = add(dir.path(), "kind=dev", &[]).failure();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        stderr.contains("hint: template `dev` files kind=dev: mt add -t dev"),
        "{stderr}"
    );
}

#[test]
fn no_hint_when_no_template_pins_the_value() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    let out = add(dir.path(), "kind=feat", &["--set", "kind=bogus"]).failure();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(!stderr.contains("hint:"), "{stderr}");
}
