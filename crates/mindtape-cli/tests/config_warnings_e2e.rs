//! A config filter naming an undeclared `x-` field warns at load, and still
//! loads: reading an undeclared name stays legal.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

mod support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.fields.x-machine]\nvalues = [\"alpha\"]\n\
     [[tasks.rules]]\nname = \"r\"\nassert = 'x-machnie == \"alpha\"'\nseverity = \"warn\"\n";

#[test]
fn a_command_warns_about_an_undeclared_extra_in_a_rule() {
    let dir = support::project_with_template_and_config(
        CONFIG,
        "#import \"@local/mindtape:0.2.0\": *\n#show: task.with(title: slot(\"title\", example: \"T\"))\n",
    );
    let out = support::mt(dir.path()).arg("ls").output().unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("warning: .mindtape/config.toml: tasks.rules[0].assert: `x-machnie`"),
        "{stderr}"
    );
    assert!(stderr.contains("did you mean x-machine?"), "{stderr}");
}
