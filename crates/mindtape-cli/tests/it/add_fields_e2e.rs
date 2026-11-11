//! End-to-end tests for writable fields `mt add` writes without a template
//! slot: the shared library writer makes `--set due=…` and `--set tags+=…`
//! work on a template that never mentions those fields.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

/// A template's hardcoded title still names the file when the caller overrides
/// it, even though no `title` slot exists to carry the value.
#[test]
fn a_set_title_without_a_title_slot_mints_the_stem() {
    let dir = support::project_with_template(mindtape::typst_task!("(title: \"Placeholder\")\n"));

    support::run_ok(dir.path(), &["add", "--set", "title=Budget"]);

    let written = std::fs::read_to_string(dir.path().join("tasks/budget.typ")).unwrap();
    assert!(written.contains("title: \"Budget\""), "{written}");
}

/// A template with a `tags` list slot whose default is not empty, so `+=` has
/// something visible to append to.
const TAGS_TEMPLATE: &str = mindtape::typst_task!(
    r#"(
  title: slot("title", example: "Tagged"),
  tags: slot("tags", kind: "list", default: ("base",)),
)
"#
);

/// A template with a literal `tags` array and no slot for it.
const LITERAL_TAGS_TEMPLATE: &str = mindtape::typst_task!(
    r#"(
  title: slot("title", example: "Tagged"),
  tags: ("base",),
)
"#
);

/// One field-write shape and what the filed task must say after it.
struct Case {
    template: Option<&'static str>,
    name: &'static str,
    sets: &'static [&'static str],
    contains: &'static [&'static str],
    absent: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        template: None,
        name: "dated",
        sets: &["title=Remove the legacy import", "due=2026-12-01"],
        contains: &["due: on(2026, 12, 1)"],
        absent: &[],
    },
    Case {
        template: Some(TAGS_TEMPLATE),
        name: "tagged",
        sets: &["title=Tagged", "tags+=bug"],
        contains: &["tags: (\"base\", \"bug\")"],
        absent: &[],
    },
    Case {
        template: Some(TAGS_TEMPLATE),
        name: "replaced",
        sets: &["title=Replaced", "tags=bug"],
        contains: &["tags: (\"bug\",)"],
        absent: &["base"],
    },
    Case {
        template: Some(TAGS_TEMPLATE),
        name: "dropped",
        sets: &["title=Dropped", "tags+=old", "tags=new"],
        contains: &["tags: (\"new\",)"],
        absent: &["old"],
    },
    Case {
        template: Some(LITERAL_TAGS_TEMPLATE),
        name: "literal",
        sets: &["title=Literal", "tags+=bug"],
        contains: &["tags: (\"base\", \"bug\")"],
        absent: &[],
    },
    Case {
        template: Some(LITERAL_TAGS_TEMPLATE),
        name: "literal-dropped",
        sets: &["title=Literal", "tags+=old", "tags=new"],
        contains: &["tags: (\"new\",)"],
        absent: &["old"],
    },
];

#[test]
fn writable_fields_apply_in_order_without_a_slot() {
    for case in CASES {
        let dir = match case.template {
            Some(template) => support::project_with_template(template),
            None => support::scaffold(),
        };
        let mut args = vec!["add", "--name", case.name];
        for set in case.sets {
            args.extend(["--set", set]);
        }
        support::run_ok(dir.path(), &args);
        let written =
            std::fs::read_to_string(dir.path().join("tasks").join(format!("{}.typ", case.name)))
                .unwrap();
        for wanted in case.contains {
            assert!(written.contains(wanted), "{}: {written}", case.name);
        }
        for unwanted in case.absent {
            assert!(!written.contains(unwanted), "{}: {written}", case.name);
        }
    }
}

#[test]
fn a_root_outside_the_project_is_refused_naming_tasks_roots() {
    let outside = format!("../mt-outside-root-{}", std::process::id());
    let config = format!("[tasks]\nroots = [\"{outside}\"]\n");
    let dir = support::project_with_template_and_config(&config, TAGS_TEMPLATE);

    let output = support::mt(dir.path())
        .args([
            "add",
            "Rooted",
            "--name",
            "outside-root",
            "--set",
            "due=2026-12-01",
        ])
        .assert()
        .failure()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("tasks.roots"), "{stderr}");
    assert!(
        !dir.path().join(&outside).join("outside-root.typ").exists(),
        "a refused field write must not land"
    );
}

#[test]
fn add_refuses_dir_with_the_add_or_rename_hint() {
    let dir = support::scaffold();

    let output = support::mt(dir.path())
        .args([
            "add",
            "--name",
            "hidden",
            "--set",
            "title=Hidden",
            "--set",
            "dir=fixes",
        ])
        .assert()
        .failure()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--dir"), "{stderr}");
    assert!(stderr.contains("mt mv only renames"), "{stderr}");
    assert!(!dir.path().join("tasks/hidden.typ").exists());
}
