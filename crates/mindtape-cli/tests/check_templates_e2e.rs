//! `mt check` files every template `mt add --list` lists with each slot's
//! `example`, else its `default`, and reports every breach of the result at
//! the template's file and line. A slot with neither cannot be filled, so it
//! is a template error naming the slot.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

mod support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.fields.kind]\nvalues = [\"fix\"]\n";

/// A project whose one template is `bug.typ`, also written as `task.typ` so
/// the built-in fallback is out of the listing.
fn scaffold(template: &str) -> tempfile::TempDir {
    scaffold_with(CONFIG, template)
}

fn scaffold_with(config: &str, template: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let templates = dir.path().join(".mindtape/templates");
    std::fs::create_dir_all(&templates).unwrap();
    std::fs::write(dir.path().join(".mindtape/config.toml"), config).unwrap();
    std::fs::write(templates.join("bug.typ"), template).unwrap();
    std::fs::write(templates.join("task.typ"), template).unwrap();
    support::install_lib(dir.path());
    dir
}

fn check(dir: &std::path::Path) -> (bool, String) {
    let out = support::mt(dir).arg("check").output().unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

fn template(kind: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.2.0\": *\n#show: task.with(\n  \
         title: slot(\"title\", example: \"Fix the crash on save\"),\n  kind: {kind},\n)\n"
    )
}

#[test]
fn a_stale_literal_fails_naming_the_template_line() {
    let dir = scaffold(&template("\"bug\""));
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(
        text.contains("error: .mindtape/templates/bug.typ:4: kind: `bug` is not an allowed value"),
        "{text}"
    );
}

#[test]
fn a_stale_example_fails() {
    let dir = scaffold(&template("slot(\"kind\", example: \"bug\")"));
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(
        text.contains(".mindtape/templates/bug.typ:4: kind: `bug` is not an allowed value"),
        "{text}"
    );
}

#[test]
fn a_stale_default_fails() {
    let dir = scaffold(&template("slot(\"kind\", default: \"bug\")"));
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(
        text.contains(".mindtape/templates/bug.typ:4: kind: `bug` is not an allowed value"),
        "{text}"
    );
}

/// The example wins over the default, so a stale default behind a good
/// example is not what the check files.
#[test]
fn the_example_fills_before_the_default() {
    let dir = scaffold(&template(
        "slot(\"kind\", default: \"fix\", example: \"bug\")",
    ));
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(text.contains("`bug` is not an allowed value"), "{text}");
}

#[test]
fn a_slot_with_neither_example_nor_default_names_the_slot() {
    let dir = scaffold(&template("slot(\"kind\")"));
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(
        text.contains(".mindtape/templates/bug.typ:4: template: slot `kind` has no example"),
        "{text}"
    );
}

/// A slot `mt add` adds for a config-required field the template omits takes
/// its example from config, and that example passes the config's own checks.
#[test]
fn a_slot_added_for_a_required_field_fills_from_config() {
    let config = format!(
        "{CONFIG}[tasks.difficulty]\nrequired = true\n\
         [tasks.priority]\nframework = \"ice\"\nallow-manual = false\nrequired = true\n\
         [tasks.estimate]\nunits = [\"src-files\"]\nrequired = true\n\
         [tasks.assignee]\nallowed = [\"max\", \"agent\"]\nrequired = \"!terminal\"\n\
         [tasks.fields.x-team]\nrequired = true\n\
         [[actors]]\nname = \"max\"\nkind = \"human\"\n\
         [[actors]]\nname = \"agent\"\nkind = \"llm\"\n"
    );
    let dir = scaffold_with(&config, &template("\"fix\""));
    let (ok, text) = check(dir.path());
    assert!(ok, "{text}");
}

#[test]
fn a_healthy_template_passes() {
    let dir = scaffold(&template("slot(\"kind\", example: \"fix\")"));
    let (ok, text) = check(dir.path());
    assert!(ok, "{text}");
}

#[test]
fn a_template_that_fails_to_evaluate_is_one_breach() {
    let dir = scaffold(&template("no-such-helper()"));
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    let breaches: Vec<&str> = text
        .lines()
        .filter(|line| line.contains(".mindtape/templates/bug.typ"))
        .collect();
    assert_eq!(breaches.len(), 1, "{text}");
    assert!(
        breaches.first().unwrap().contains("no-such-helper"),
        "{text}"
    );
}

/// A body slot's raw-block example is checked by the body rules like a
/// literal body.
#[test]
fn a_body_rule_reads_the_body_s_example() {
    let rule = "[[tasks.body.headers.rules]]\nname = \"only-problem\"\n\
                action = \"allowlist\"\nallowed = [{ match = \"^Problem$\" }]\n";
    let body = |heading: &str| {
        format!(
            "{}\n#slot(\"body\", kind: \"raw\", example: ```typ\n= {heading}\n\nIt crashes.\n```)\n",
            template("\"fix\"")
        )
    };
    let dir = scaffold_with(&format!("{CONFIG}{rule}"), &body("Problem"));
    let (ok, text) = check(dir.path());
    assert!(ok, "{text}");

    let dir = scaffold_with(&format!("{CONFIG}{rule}"), &body("Context"));
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(
        text.contains(".mindtape/templates/bug.typ:8: body.headers.rules.only-problem"),
        "{text}"
    );
}

fn rule(name: &str, assert: &str) -> String {
    format!(
        "{CONFIG}[[tasks.rules]]\nname = \"{name}\"\nassert = '{assert}'\nseverity = \"error\"\n"
    )
}

/// Every breach of the filled template counts, a rule's included.
#[test]
fn a_rule_breach_on_an_example_counts() {
    let dir = scaffold_with(
        &rule("no-bare-fix", "kind != \"fix\""),
        &template("slot(\"kind\", example: \"fix\")"),
    );
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(
        text.contains(".mindtape/templates/bug.typ:2: rules.no-bare-fix"),
        "{text}"
    );
}

/// A template's own `status:` names the status `mt add` files it at, so a
/// rule that holds a template's tasks to that status passes the check.
#[test]
fn a_template_status_is_the_status_it_is_filed_at() {
    let filed_done = template("\"fix\"").replace("  kind:", "  status: done(2026, 1, 1),\n  kind:");
    let dir = scaffold_with(&rule("lands-done", "terminal"), &filed_done);
    let (ok, text) = check(dir.path());
    assert!(ok, "{text}");
}

/// A project with no `task.typ` gets the built-in template, whose breaches
/// warn unless `[templates] fallback-severity` says `error`.
fn fallback_project(extra: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    let config = format!("{}{extra}", rule("dated", "has(due)"));
    std::fs::write(dir.path().join(".mindtape/config.toml"), config).unwrap();
    support::install_lib(dir.path());
    dir
}

#[test]
fn the_built_in_template_warns_by_default() {
    let dir = fallback_project("");
    let (ok, text) = check(dir.path());
    assert!(ok, "{text}");
    assert!(
        text.contains("warning: built-in template task:3: rules.dated"),
        "{text}"
    );
}

#[test]
fn fallback_severity_error_fails_on_the_built_in_template() {
    let dir = fallback_project("[templates]\nfallback-severity = \"error\"\n");
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(
        text.contains("error: built-in template task:3: rules.dated"),
        "{text}"
    );
}

/// The key is settable in the global defaults layer like any other.
#[test]
fn fallback_severity_reads_the_global_defaults_layer() {
    let dir = fallback_project("");
    let (_, home) = support::isolated_env(dir.path())
        .into_iter()
        .find(|(key, _)| *key == mindtape::CONFIG_HOME_ENV)
        .unwrap();
    std::fs::create_dir_all(home.join("mindtape")).unwrap();
    std::fs::write(
        home.join("mindtape/defaults.toml"),
        "[templates]\nfallback-severity = \"error\"\n",
    )
    .unwrap();
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(
        text.contains("error: built-in template task:3: rules.dated"),
        "{text}"
    );
}
