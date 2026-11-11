//! End-to-end tests for `[[on.add]]`: the draft task a hook reads, the ordered
//! set/append/dir actions, explicit input winning, the outside-root refusals
//! and the describe surface.
#![allow(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing
)]

use super::add_hooks_support::{BASE, FILL_AND_DIR, TEMPLATE, add, project, read};
use crate::support;
#[test]
fn hooks_fill_fields_and_choose_the_destination_in_order() {
    let dir = project(FILL_AND_DIR);
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "lesson", "Fix the login"]);
    assert!(ok, "{stderr}");
    assert!(stderr.contains("on.add fixes: set kind"), "{stderr}");
    assert!(
        stderr.contains("on.add regression: append tags"),
        "{stderr}"
    );
    let written = read(dir.path(), "tasks/bug/lesson.typ");
    assert!(written.contains("kind: \"bug\""), "{written}");
    assert!(written.contains("due: on(2026, 12, 1)"), "{written}");
    assert!(
        written.contains("tags: (\"base\", \"regression\")"),
        "{written}"
    );
}

#[test]
fn an_explicit_set_owns_the_field_and_the_dir_follows_it() {
    let dir = project(FILL_AND_DIR);
    let (_, stderr, ok) = add(
        dir.path(),
        &[
            "add",
            "--name",
            "explicit",
            "--set",
            "kind=feat",
            "Explicit wins",
        ],
    );
    assert!(ok, "{stderr}");
    assert!(stderr.contains("kind keeps the explicit value"), "{stderr}");
    let written = read(dir.path(), "tasks/feat/explicit.typ");
    assert!(written.contains("kind: \"feat\""), "{written}");
    assert!(!written.contains("kind: \"bug\""), "{written}");
}

#[test]
fn a_hook_append_reaches_a_slot_the_caller_set() {
    let dir = project(FILL_AND_DIR);
    let (_, stderr, ok) = add(
        dir.path(),
        &["add", "--name", "tagged", "--set", "tags=seed", "Tagged"],
    );
    assert!(ok, "{stderr}");
    let written = read(dir.path(), "tasks/bug/tagged.typ");
    assert!(written.contains("\"seed\""), "{written}");
    assert!(written.contains("\"regression\""), "{written}");
}

#[test]
fn a_missing_required_title_is_refused_with_hooks_on() {
    let dir = project(FILL_AND_DIR);
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "untitled"]);
    assert!(!ok, "{stderr}");
    assert!(
        stderr.contains("missing a required slot: 'title'"),
        "{stderr}"
    );
    assert!(!dir.path().join("tasks/bug/untitled.typ").exists());
}

/// The positional title is the caller's own input, so a hook `set title`
/// leaves it alone, as an explicit `--set title` does.
#[test]
fn a_positional_title_keeps_a_hook_from_setting_it() {
    let dir = project(
        "[[on.add]]\nname = \"retitle\"\nactions = [{ set = { title = \"From the hook\" } }]\n",
    );
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "titled", "From argv"]);
    assert!(ok, "{stderr}");
    assert!(
        stderr.contains("title keeps the explicit value"),
        "{stderr}"
    );
    let written = read(dir.path(), "tasks/titled.typ");
    assert!(written.contains("title: \"From argv\""), "{written}");
    assert!(!written.contains("From the hook"), "{written}");
}

#[test]
fn an_explicit_dir_wins_over_a_hook_dir() {
    let dir = project(FILL_AND_DIR);
    let (_, stderr, ok) = add(
        dir.path(),
        &["add", "--dir", "tasks/custom", "--name", "placed", "Placed"],
    );
    assert!(ok, "{stderr}");
    assert!(read(dir.path(), "tasks/custom/placed.typ").contains("kind: \"bug\""));
}

#[test]
fn an_explicit_dir_does_not_expand_a_hook_dir() {
    let dir = project("[[on.add]]\nname = \"by-kind\"\nactions = [{ dir = \"tasks/{kind}\" }]\n");
    let (_, stderr, ok) = add(
        dir.path(),
        &[
            "add",
            "--dir",
            "tasks/custom",
            "--name",
            "skipped",
            "Skipped",
        ],
    );
    assert!(ok, "{stderr}");
    assert!(read(dir.path(), "tasks/custom/skipped.typ").contains("title: \"Skipped\""));
}

/// A configured root may not exist yet: it is still the root a hook's `dir`
/// resolves under, or the first add into it would refuse itself.
#[test]
fn a_hook_dir_inside_a_root_that_does_not_exist_yet_is_accepted() {
    let config = BASE.replace("roots = [\"tasks\"]", "roots = [\"backlog\"]");
    let extra = "[[on.add]]\nname = \"nested\"\nactions = [{ dir = \"backlog/fixes\" }]\n";
    let dir = support::scaffold_project(&format!("{config}{extra}"));
    std::fs::create_dir_all(dir.path().join(".mindtape/templates")).unwrap();
    std::fs::write(dir.path().join(".mindtape/templates/task.typ"), TEMPLATE).unwrap();
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "fresh", "Fresh"]);
    assert!(ok, "{stderr}");
    assert!(read(dir.path(), "backlog/fixes/fresh.typ").contains("title: \"Fresh\""));
}

/// A `+tag` positional is the base an explicit `--set tags=…` replaces, and an
/// explicit `+=` appends to it without repeating the same tag.
#[test]
fn the_positional_tags_are_the_base_of_the_set_tokens() {
    let dir = project("");
    let (_, stderr, ok) = add(
        dir.path(),
        &[
            "add", "--name", "replaced", "+auth", "--set", "tags=bug", "Replaced",
        ],
    );
    assert!(ok, "{stderr}");
    let written = read(dir.path(), "tasks/replaced.typ");
    assert!(written.contains("tags: (\"bug\",)"), "{written}");
    assert!(!written.contains("auth"), "{written}");

    let (_, stderr, ok) = add(
        dir.path(),
        &[
            "add",
            "--name",
            "deduped",
            "+auth",
            "--set",
            "tags+=auth",
            "Deduped",
        ],
    );
    assert!(ok, "{stderr}");
    let written = read(dir.path(), "tasks/deduped.typ");
    assert!(written.contains("tags: (\"base\", \"auth\")"), "{written}");
}

#[test]
fn no_hooks_skips_them_but_keeps_explicit_fields() {
    let dir = project(FILL_AND_DIR);
    let (_, stderr, ok) = add(
        dir.path(),
        &[
            "add",
            "--no-hooks",
            "--name",
            "plain",
            "--set",
            "assignee=ada",
            "Plain",
        ],
    );
    assert!(ok, "{stderr}");
    assert!(!stderr.contains("on.add"), "{stderr}");
    let written = read(dir.path(), "tasks/plain.typ");
    assert!(written.contains("assignee: \"ada\""), "{written}");
    assert!(!written.contains("due:"), "{written}");
}

#[test]
fn a_dir_rule_refuses_a_misplaced_hook_destination() {
    let dir = project(&format!(
        "{FILL_AND_DIR}
[[tasks.rules]]
name = \"bugs-in-their-dir\"
severity = \"error\"
when = 'kind == \"bug\"'
assert = 'dir == \"tasks/fixes\"'
"
    ));
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "misplaced", "Misplaced"]);
    assert!(!ok, "{stderr}");
    assert!(stderr.contains("bugs-in-their-dir"), "{stderr}");
    assert!(
        !dir.path().join("tasks/bug/misplaced.typ").exists(),
        "a refused task must not land"
    );
}

#[test]
fn a_path_like_expansion_is_refused() {
    let dir = project("[[on.add]]\nname = \"by-kind\"\nactions = [{ dir = \"tasks/{kind}\" }]\n");
    let (_, stderr, ok) = add(
        dir.path(),
        &["add", "--name", "slash", "--set", "kind=a/b", "Slash"],
    );
    assert!(!ok, "{stderr}");
    assert!(stderr.contains("path separator"), "{stderr}");
    assert!(!dir.path().join("tasks/slash.typ").exists());
}

#[test]
fn an_unknown_dir_macro_is_a_config_error() {
    let dir = project("[[on.add]]\nname = \"bad\"\nactions = [{ dir = \"tasks/{nope}\" }]\n");
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "bad", "Bad"]);
    assert!(!ok, "{stderr}");
    assert!(stderr.contains("unknown dir macro"), "{stderr}");
}

#[test]
fn a_hooked_add_refuses_a_destination_outside_every_root() {
    let dir = project("[[on.add]]\nname = \"escape\"\nactions = [{ dir = \"../outside\" }]\n");
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "escape", "Escape"]);
    assert!(!ok, "{stderr}");
    assert!(stderr.contains("outside the configured roots"), "{stderr}");
}

#[test]
fn a_hooked_explicit_dir_outside_every_root_is_refused_before_the_draft() {
    let dir = project(FILL_AND_DIR);
    let (_, stderr, ok) = add(
        dir.path(),
        &[
            "add",
            "--dir",
            "/tmp/mt-hooks-outside",
            "--name",
            "escape",
            "Escape",
        ],
    );
    assert!(!ok, "{stderr}");
    assert!(stderr.contains("outside the configured roots"), "{stderr}");
}

#[test]
fn the_draft_is_not_written_and_its_derived_tags_do_not_fire_a_hook() {
    let config = r#"
[[tasks.roots]]
path = "tasks"
name = "sandbox"

[tasks.derive-tags]
enabled = true
collect = ["segments"]
"#;
    let extra = format!(
        "{config}
[[on.add]]
name = \"root-tag\"
when = 'has_tag(\"sandbox\")'
actions = [{{ set = {{ kind = \"bug\" }} }}]
"
    );
    let dir = support::scaffold_project(&extra);
    std::fs::create_dir_all(dir.path().join(".mindtape/templates")).unwrap();
    std::fs::write(dir.path().join(".mindtape/templates/task.typ"), TEMPLATE).unwrap();
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "clean", "Clean"]);
    assert!(ok, "{stderr}");
    assert!(!read(dir.path(), "tasks/clean.typ").contains("kind:"));
    assert!(!dir.path().join("tasks/__draft__.typ").exists());
}

#[test]
fn describe_names_the_hooks_and_uses_the_new_schema() {
    let dir = project(FILL_AND_DIR);
    let (stdout, stderr, ok) = add(dir.path(), &["add", "--describe=json"]);
    assert!(ok, "{stderr}");
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(
        json["$schema"]
            .as_str()
            .is_some_and(|url| url.ends_with("/describe/2.schema.json")),
        "{stdout}"
    );
    assert_eq!(json["hooks"][0]["name"], "fixes");
    assert!(
        json["hooks"][0]["actions"][0]
            .as_str()
            .is_some_and(|line| line.contains("dir = \"tasks/{kind}\"")),
        "{stdout}"
    );
}
#[test]
fn a_hook_set_of_a_number_reaches_an_int_slot() {
    let dir = project("[[on.add]]\nname = \"prio\"\nactions = [{ set = { priority = 3 } }]\n");
    std::fs::write(
        dir.path().join(".mindtape/templates/task.typ"),
        mindtape::typst_task!(
            r#"(
  title: slot("title"),
  priority: slot("priority", kind: "int"),
)
"#
        ),
    )
    .unwrap();
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "numbered", "Numbered"]);
    assert!(ok, "{stderr}");
    assert!(read(dir.path(), "tasks/numbered.typ").contains("priority: 3"));
}

/// A template that names a core field through a slot under another name
/// (`kind: slot("type")`): the caller's explicit value owns the field by its
/// core name too, so a hook `set kind` keeps its hands off.
#[test]
fn an_explicit_value_under_an_aliased_slot_owns_the_field() {
    let dir =
        project("[[on.add]]\nname = \"classify\"\nactions = [{ set = { kind = \"feat\" } }]\n");
    std::fs::write(
        dir.path().join(".mindtape/templates/task.typ"),
        mindtape::typst_task!(
            r#"(
  title: slot("title"),
  kind: slot("type"),
)
"#
        ),
    )
    .unwrap();

    let (_, stderr, ok) = add(
        dir.path(),
        &["add", "--name", "aliased", "--set", "type=bug", "Aliased"],
    );

    assert!(ok, "{stderr}");
    assert!(stderr.contains("kind keeps the explicit value"), "{stderr}");
    let written = read(dir.path(), "tasks/aliased.typ");
    assert!(written.contains("kind: \"bug\""), "{written}");
    assert!(!written.contains("kind: \"feat\""), "{written}");
}

/// A hook append into a composite tags expression replaces the whole argument,
/// so the literal prefix is not duplicated by the final fill.
#[test]
fn a_hook_append_reaches_a_composite_tags_expression_once() {
    let dir = project(
        "[[on.add]]\nname = \"regression\"\nactions = [{ append = { tags = [\"regression\"] } }]\n",
    );
    std::fs::write(
        dir.path().join(".mindtape/templates/task.typ"),
        mindtape::typst_task!(
            r#"(
  title: slot("title"),
  tags: ("base",) + slot("tags", kind: "list", default: ()),
)
"#
        ),
    )
    .unwrap();

    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "composite", "Composite"]);

    assert!(ok, "{stderr}");
    let written = read(dir.path(), "tasks/composite.typ");
    assert!(
        written.contains("tags: (\"base\", \"regression\")"),
        "{written}"
    );
    assert_eq!(written.matches("\"base\"").count(), 1, "{written}");
}

/// A strict priority has no scalar form, so `--priority` is refused; the
/// refusal names the manual priority rather than the framework leaves the
/// replaced expression used to leave unfilled.
#[test]
fn a_strict_framework_refuses_a_manual_priority_by_name() {
    let dir =
        project("[tasks.priority]\nframework = \"ice\"\nallow-manual = false\nrequired = true\n");

    let (_, stderr, ok) = add(
        dir.path(),
        &["add", "--name", "strict", "--priority", "6", "Strict"],
    );

    assert!(!ok, "{stderr}");
    assert!(!stderr.contains("missing a required slot"), "{stderr}");
    assert!(stderr.contains("manual priority"), "{stderr}");
}

/// A hook's `set title` mints the file stem too, even when the template writes
/// a hardcoded title instead of a title slot.
#[test]
fn a_hook_set_title_mints_the_stem_without_a_title_slot() {
    let dir =
        project("[[on.add]]\nname = \"retitle\"\nactions = [{ set = { title = \"Budget\" } }]\n");
    std::fs::write(
        dir.path().join(".mindtape/templates/task.typ"),
        mindtape::typst_task!("(title: \"Placeholder\")\n"),
    )
    .unwrap();

    let (_, stderr, ok) = add(dir.path(), &["add"]);

    assert!(ok, "{stderr}");
    let written = read(dir.path(), "tasks/budget.typ");
    assert!(written.contains("title: \"Budget\""), "{written}");
}

/// A hook-filled required priority is the hook's value, not config's: the
/// "filled in from config" note stays quiet for a field a hook touched.
#[test]
fn a_hook_set_priority_is_not_reported_as_filled_from_config() {
    let dir = project(
        "[tasks.priority]\nrequired = true\n\n[[on.add]]\nname = \"prio\"\nactions = [{ set = { priority = 3 } }]\n",
    );

    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "hooked", "Hooked"]);

    assert!(ok, "{stderr}");
    assert!(!stderr.contains("filled in from config"), "{stderr}");
    assert!(read(dir.path(), "tasks/hooked.typ").contains("priority: 3"));
}

#[test]
fn old_reads_the_value_before_any_hook_ran() {
    let extra = r#"
[[on.add]]
name = "change-kind"
actions = [{ set = { kind = "bug" } }]

[[on.add]]
name = "regression"
when = 'old("kind") == "feat" && kind == "bug"'
actions = [{ append = { tags = ["regression"] } }]
"#;
    let dir = support::scaffold_project(&format!("{BASE}{extra}"));
    std::fs::create_dir_all(dir.path().join(".mindtape/templates")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape/templates/task.typ"),
        mindtape::typst_task!(
            r#"(
  title: slot("title"),
  kind: "feat",
  tags: slot("tags", kind: "list", default: ("base",)),
)
"#
        ),
    )
    .unwrap();
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "regression", "Old value"]);
    assert!(ok, "{stderr}");
    assert!(
        stderr.contains("on.add regression: append tags"),
        "{stderr}"
    );
    let written = read(dir.path(), "tasks/regression.typ");
    assert!(written.contains("kind: \"bug\""), "{written}");
    assert!(
        written.contains("tags: (\"base\", \"regression\")"),
        "{written}"
    );
}
