//! A config filter naming an undeclared `x-` field warns at load, and still
//! loads: reading an undeclared name stays legal.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.fields.x-machine]\nvalues = [\"alpha\"]\n\
     [[tasks.rules]]\nname = \"r\"\nassert = 'x-machnie == \"alpha\"'\nseverity = \"warn\"\n";

/// `mt ls` over a project with `config`, and `pack` as its `team.toml`: what
/// it printed to stderr.
fn ls_stderr(config: &str, pack: &str) -> String {
    let dir = support::project_with_template_and_config(
        config,
        mindtape::typst_task!("(title: slot(\"title\", example: \"T\"))\n"),
    );
    std::fs::write(dir.path().join("team.toml"), pack).unwrap();
    let out = support::mt(dir.path()).arg("ls").output().unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn a_command_warns_about_an_undeclared_extra_in_a_rule() {
    let stderr = ls_stderr(CONFIG, "");
    assert!(
        stderr.contains(&shown(
            "warning: .mindtape/config.toml:7: tasks.rules[0].assert: `x-machnie`"
        )),
        "{stderr}"
    );
    assert!(stderr.contains("did you mean x-machine?"), "{stderr}");
}

#[test]
fn a_warning_from_a_pack_points_into_the_pack() {
    let own = "extends = [\"./team.toml\"]\n[tasks]\nroots = [\"tasks\"]\n";
    let pack = CONFIG.trim_start_matches("[tasks]\nroots = [\"tasks\"]\n");
    let stderr = ls_stderr(own, pack);
    assert!(
        stderr.contains("warning: team.toml:5: tasks.rules[0].assert: `x-machnie`"),
        "{stderr}"
    );
}

/// An expected message, each task path in it in the separator `mt` shows.
fn shown(text: &str) -> String {
    support::native_in(text, &[".mindtape/config.toml"])
}
