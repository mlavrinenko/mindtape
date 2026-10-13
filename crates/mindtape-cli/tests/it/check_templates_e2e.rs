//! `mt check` files every template `mt add --list` lists with each slot's
//! `example`, else its `default`, and reports every breach of the result at
//! the template's file and line. A slot with neither cannot be filled, so it
//! is a template error naming the slot.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;
use mindtape::test_support::tempdir;
use mindtape::test_support::write_file;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.fields.kind]\nvalues = [\"fix\"]\n";

/// A project whose one template is `bug.typ`, also written as `task.typ` so
/// the built-in fallback is out of the listing.
fn scaffold(template: &str) -> tempfile::TempDir {
    scaffold_with(CONFIG, template)
}

fn scaffold_with(config: &str, template: &str) -> tempfile::TempDir {
    let dir = tempdir();
    let templates = dir.path().join(".mindtape/templates");
    std::fs::create_dir_all(&templates).unwrap();
    write_file(dir.path(), ".mindtape/config.toml", config);
    write_file(&templates, "bug.typ", template);
    write_file(&templates, "task.typ", template);
    support::install_lib(dir.path());
    dir
}

fn check(dir: &std::path::Path) -> (bool, String) {
    support::run_text(dir, &["check"])
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

/// A `text` slot's example that shows a code span files as backslashes once
/// `mt add` escapes it, so the check names the slot at its line.
#[test]
fn a_text_slot_example_with_markup_names_the_slot() {
    let body = "\n#slot(\"why\", kind: \"text\", example: \"the `retire` verb\")\n";
    let dir = scaffold(&format!("{}{body}", template("\"fix\"")));
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(
        text.contains(
            ".mindtape/templates/bug.typ:7: template: slot `why` is `text`, so `mt add` files the code span"
        ),
        "{text}"
    );
}

/// A slot `mt add` adds for a config-required field the template omits takes
/// its example from config, and that example passes the config's own checks.
#[test]
fn a_slot_added_for_a_required_field_fills_from_config() {
    let config = format!(
        "{CONFIG}[tasks.difficulty]\nrequired = true\nexample = 5\n\
         [tasks.priority]\nframework = \"ice\"\nallow-manual = false\nrequired = true\n\
         [tasks.estimate]\nunits = [\"src-files\"]\nrequired = true\nexample = {{ src-files = 2 }}\n\
         [tasks.assignee]\nallowed = [\"max\", \"agent\"]\nrequired = \"!terminal\"\nexample = \"max\"\n\
         [tasks.fields.x-team]\nrequired = true\nexample = \"core\"\n\
         [[actors]]\nname = \"max\"\nkind = \"human\"\n\
         [[actors]]\nname = \"agent\"\nkind = \"llm\"\n"
    );
    let dir = scaffold_with(&config, &template("\"fix\""));
    let (ok, text) = check(dir.path());
    assert!(ok, "{text}");
}

/// A field whose shape a rule holds: only a configured example fills it.
fn ticket(example: &str) -> String {
    format!(
        "{CONFIG}[tasks.fields.x-ticket]\nrequired = true\n{example}\
         [[tasks.rules]]\nname = \"ticket-shape\"\n\
         assert = 'matches(x-ticket, \"^[A-Z]+-[0-9]+$\")'\nseverity = \"error\"\n"
    )
}

#[test]
fn a_configured_example_fills_a_spliced_slot() {
    let config = ticket("example = \"ABC-1\"\n");
    let dir = scaffold_with(&config, &template("\"fix\""));
    let (ok, text) = check(dir.path());
    assert!(ok, "{text}");
}

#[test]
fn a_spliced_slot_without_an_example_names_its_config_path() {
    let config = ticket("");
    let dir = scaffold_with(&config, &template("\"fix\""));
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(
        text.contains("slot `x-ticket` has no example; set `example` in [tasks.fields.x-ticket]"),
        "{text}"
    );
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
    assert!(
        text.contains("error: .mindtape/templates/bug.typ:4:9: unknown variable: no-such-helper"),
        "the load failure reads as a task file's does: {text}"
    );
}

/// Typst's hints follow a template's load failure as `hint:` lines.
#[test]
fn a_template_load_failure_keeps_typsts_hints() {
    let dir = scaffold(&template("some-name-here"));
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(
        text.contains("unknown variable: some-name-here\nhint: "),
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
    support::scaffold_project(&format!("{}{extra}", rule("dated", "has(due)")))
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
    write_file(
        &home,
        "mindtape/defaults.toml",
        "[templates]\nfallback-severity = \"error\"\n",
    );
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(
        text.contains("error: built-in template task:3: rules.dated"),
        "{text}"
    );
}

/// A task file at `tasks/<stem>.typ`, valid under [`CONFIG`].
fn add_task(dir: &std::path::Path, stem: &str, title: &str) {
    let file = dir.join(format!("tasks/{stem}.typ"));
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let source = format!(
        "#import \"@local/mindtape:0.2.0\": *\n#show: task.with(\n  title: \"{title}\",\n  \
         kind: \"fix\",\n  status: proposed(2026, 1, 1),\n)\n"
    );
    std::fs::write(file, source).unwrap();
}

/// A cap the backlog holds is not the template's to break: each template is
/// checked alone, against no tasks, and a breach that reads the whole set is
/// not one the template can fix.
#[test]
fn a_backlog_at_its_count_cap_does_not_fail_a_template() {
    let dir = scaffold_with(
        &rule("open-cap", "count(!terminal) <= 4"),
        &template("\"fix\""),
    );
    for index in 0..4 {
        add_task(
            dir.path(),
            &format!("open-{index}"),
            &format!("Open {index}"),
        );
    }
    let (ok, text) = check(dir.path());
    assert!(ok, "{text}");
}

/// A cap a template alone would break reads the set, so it is dropped too: the
/// template cannot fix it.
#[test]
fn a_count_cap_of_zero_is_not_a_template_breach() {
    let dir = scaffold_with(
        &rule("none-open", "count(!terminal) <= 0"),
        &template("\"fix\""),
    );
    let (ok, text) = check(dir.path());
    assert!(ok, "{text}");
}

/// A rule that reads no set still counts on the template alone.
#[test]
fn a_rule_that_reads_no_set_still_counts_beside_a_set_rule() {
    let config = format!(
        "{}[[tasks.rules]]\nname = \"no-fix\"\nassert = 'kind != \"fix\"'\nseverity = \"error\"\n",
        rule("open-cap", "count(!terminal) <= 0")
    );
    let dir = scaffold_with(&config, &template("\"fix\""));
    let (ok, text) = check(dir.path());
    assert!(!ok, "{text}");
    assert!(text.contains("rules.no-fix"), "{text}");
    assert!(!text.contains("rules.open-cap"), "{text}");
}

/// The candidate is named from the example title, where `mt add` would write
/// it, so a task named like the template collides with nothing.
#[test]
fn a_task_named_like_a_template_collides_with_nothing() {
    let dir = scaffold(&template("\"fix\""));
    add_task(dir.path(), "archive/bug", "Some other task");
    add_task(dir.path(), "archive/task", "Another task");
    let (ok, text) = check(dir.path());
    assert!(ok, "{text}");
}

/// A template may link to a task of the backlog; checked alone it has no
/// backlog to resolve the target in, which is not the template's breach.
#[test]
fn a_link_to_a_backlog_task_is_not_a_template_breach() {
    let linked = template("\"fix\"").replace(
        "  kind:",
        "  links: related(\"anchor.typ\")[same session],\n  kind:",
    );
    let dir = scaffold(&linked);
    add_task(dir.path(), "anchor", "Anchor");
    let (ok, text) = check(dir.path());
    assert!(ok, "{text}");
}
