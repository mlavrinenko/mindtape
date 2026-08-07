//! Unit tests for the `add` command's pure helpers: status/header injection,
//! title and tag construction, per-kind markup escaping, output-stem naming,
//! dest-dir resolution, and the outside-roots warning.
//!
//! Binary-invoking end-to-end tests live in `tests/add_e2e.rs` (integration
//! tests), where Cargo guarantees the `mt` binary is built first. Every test
//! that needs its own process — a controllable stdin, cwd, or captured
//! stderr — lives in the sibling `add_tests_subprocess` module; nothing in
//! this file spawns a child.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mindtape::{Config, RawValue, SlotSpec};

use super::{
    AddArgs, dest_dir, fill_title_from_args, initial_status, inject_header, markup_safe,
    output_stem,
};
use crate::add_output::Format;
use crate::util::Project;

const HEADER: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Do thing",
)

== Summary
"#;

fn bare_args() -> AddArgs {
    AddArgs {
        title: vec![],
        template: None,
        values: vec![],
        set: vec![],
        into: ".".to_owned(),
        to: None,
        name: None,
        status: None,
        edit: false,
        describe: None,
        list: false,
        print: false,
        no_verify: false,
        dedup: None,
        format: Format::Text,
    }
}

#[test]
fn injects_default_status_proposed() {
    // The embedded floor's legend leads with `proposed`; the initial status is
    // derived from it, not a hardcoded literal.
    let config = mindtape::parse_config("").unwrap();
    let out = inject_header(HEADER, &config, None).unwrap();
    assert!(out.contains("status: proposed("), "{out}");
    assert!(!out.contains("status(\"proposed\""), "{out}");
}

#[test]
fn leaves_status_unfilled_when_legend_is_empty() {
    // A project that clears the legend (`status = []`) gets no invented
    // `"proposed"` — the status arg is left out entirely, and `mt check` will
    // report the real "missing required status" problem.
    let out = inject_header(HEADER, &Config::default(), None).unwrap();
    assert!(!out.contains("status:"), "{out}");
}

#[test]
fn status_override_wins() {
    let out = inject_header(HEADER, &Config::default(), Some("wip")).unwrap();
    assert!(out.contains("status: wip("), "{out}");
}

#[test]
fn custom_status_falls_back_to_verbose_form() {
    // A name the prelude ships no shortcut for keeps the `status("…", …)` form.
    let out = inject_header(HEADER, &Config::default(), Some("icebox")).unwrap();
    assert!(out.contains("status: status(\"icebox\","), "{out}");
}

#[test]
fn initial_status_is_first_non_terminal() {
    let config: Config = mindtape::parse_config(
        "[[tasks.status]]\nvalue = \"done\"\nterminal = true\n\
         [[tasks.status]]\nvalue = \"open\"\n",
    )
    .unwrap();
    assert_eq!(initial_status(&config).as_deref(), Some("open"));
}

#[test]
fn output_stem_prefers_name_then_title_slug() {
    let config = Config::default();
    let dest = Path::new(".");
    let mut args = bare_args();
    args.name = Some("explicit".to_owned());
    assert_eq!(
        output_stem(&args, &BTreeMap::new(), &config, dest).unwrap(),
        "explicit"
    );

    let args = bare_args();
    let mut values = BTreeMap::new();
    values.insert("title".to_owned(), RawValue::Str("My New Task".to_owned()));
    assert_eq!(
        output_stem(&args, &values, &config, dest).unwrap(),
        "my-new-task"
    );
}

/// A minimal `tags` list slot, as a real template would discover it.
fn tags_slot() -> SlotSpec {
    mindtape::discover_slots(r#"#f(tags: slot("tags", kind: "list"))"#)
        .into_iter()
        .next()
        .unwrap()
}

#[test]
fn title_arg_joins_words_when_unset() {
    let mut args = bare_args();
    args.title = vec!["cleanup".to_owned(), "kitchen".to_owned()];
    let mut values = BTreeMap::new();
    fill_title_from_args(&mut values, &args, &[]);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("cleanup kitchen".to_owned()))
    );
}

#[test]
fn title_arg_yields_to_higher_precedence_title() {
    let mut args = bare_args();
    args.title = vec!["positional".to_owned()];
    let mut values = BTreeMap::new();
    values.insert("title".to_owned(), RawValue::Str("explicit".to_owned()));
    fill_title_from_args(&mut values, &args, &[]);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("explicit".to_owned()))
    );
}

#[test]
fn plus_tag_words_move_to_tags_when_slot_present() {
    let mut args = bare_args();
    args.title = vec![
        "fix".to_owned(),
        "login".to_owned(),
        "+backend".to_owned(),
        "+auth".to_owned(),
    ];
    let mut values = BTreeMap::new();
    fill_title_from_args(&mut values, &args, &[tags_slot()]);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("fix login".to_owned()))
    );
    assert_eq!(
        values.get("tags"),
        Some(&RawValue::List(vec![
            "backend".to_owned(),
            "auth".to_owned()
        ]))
    );
}

#[test]
fn plus_tag_words_stay_in_title_without_tags_slot() {
    let mut args = bare_args();
    args.title = vec!["fix".to_owned(), "+backend".to_owned()];
    let mut values = BTreeMap::new();
    fill_title_from_args(&mut values, &args, &[]);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("fix +backend".to_owned()))
    );
    assert!(!values.contains_key("tags"));
}

#[test]
fn plus_tag_sugar_appends_to_existing_tags() {
    let mut args = bare_args();
    args.title = vec!["fix".to_owned(), "+auth".to_owned()];
    let mut values = BTreeMap::new();
    values.insert(
        "tags".to_owned(),
        RawValue::List(vec!["backend".to_owned()]),
    );
    fill_title_from_args(&mut values, &args, &[tags_slot()]);
    assert_eq!(
        values.get("tags"),
        Some(&RawValue::List(vec![
            "backend".to_owned(),
            "auth".to_owned()
        ]))
    );
}

#[test]
fn bare_plus_stays_in_title_when_tags_slot_present() {
    let mut args = bare_args();
    args.title = vec!["fix".to_owned(), "+".to_owned(), "login".to_owned()];
    let mut values = BTreeMap::new();
    fill_title_from_args(&mut values, &args, &[tags_slot()]);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("fix + login".to_owned())),
    );
    assert!(!values.contains_key("tags"));
}

#[test]
fn output_stem_errors_without_name_or_title() {
    let config = Config::default();
    assert!(output_stem(&bare_args(), &BTreeMap::new(), &config, Path::new(".")).is_err());
}

/// The `title` value shared by the naming-template tests below.
fn ship_it_values() -> BTreeMap<String, RawValue> {
    let mut values = BTreeMap::new();
    values.insert("title".to_owned(), RawValue::Str("Ship it".to_owned()));
    values
}

/// A tempdir that already has `<stem>.typ` in it, for collision-guard tests.
fn dir_with_existing_stem(stem: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(format!("{stem}.typ")), "").unwrap();
    dir
}

#[test]
fn output_stem_applies_naming_template_when_configured() {
    let mut config = Config::default();
    config.tasks.naming.stem = Some("{slug}.task-{yy}{mm}{dd}".to_owned());
    let dir = tempfile::tempdir().unwrap();

    let stem = output_stem(&bare_args(), &ship_it_values(), &config, dir.path()).unwrap();
    assert!(stem.starts_with("ship-it.task-"), "{stem}");
    assert_eq!(stem.len(), "ship-it.task-".len() + 6);
}

#[test]
fn output_stem_disambiguates_a_templated_collision() {
    let mut config = Config::default();
    config.tasks.naming.stem = Some("{slug}".to_owned());
    let dir = dir_with_existing_stem("ship-it");

    let stem = output_stem(&bare_args(), &ship_it_values(), &config, dir.path()).unwrap();
    assert_eq!(stem, "ship-it-2");
}

#[test]
fn output_stem_unset_naming_keeps_bare_slug_even_with_existing_file() {
    // Unset `[tasks.naming]` is exactly today's behaviour: no disambiguation.
    // `run()`'s own `file.exists()` check is what turns this into an error.
    let config = Config::default();
    let dir = dir_with_existing_stem("ship-it");

    let stem = output_stem(&bare_args(), &ship_it_values(), &config, dir.path()).unwrap();
    assert_eq!(stem, "ship-it");
}

/// A `Project` rooted at `dir` whose single configured root is `root`.
fn project_with_root(dir: &Path, root: &str) -> Project {
    let mut config = Config::default();
    config.tasks.roots = vec![mindtape::RootSpec::Short(root.to_owned())];
    Project {
        root: dir.to_path_buf(),
        config,
        outcome: mindtape::LoadOutcome::default(),
    }
}

#[test]
fn dest_default_is_first_root() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_with_root(dir.path(), "truth/tasks");
    assert_eq!(
        dest_dir(".", None, &project).unwrap(),
        dir.path().join("truth/tasks")
    );
}

#[test]
fn dest_to_nests_subfolder_under_root() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_with_root(dir.path(), "truth/tasks");
    assert_eq!(
        dest_dir(".", Some("shopping"), &project).unwrap(),
        dir.path().join("truth/tasks/shopping")
    );
    // Nested subpaths are allowed.
    assert_eq!(
        dest_dir(".", Some("shopping/weekly"), &project).unwrap(),
        dir.path().join("truth/tasks/shopping/weekly")
    );
}

#[test]
fn dest_to_rejects_absolute_subfolder() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_with_root(dir.path(), "tasks");
    assert!(dest_dir(".", Some("/etc"), &project).is_err());
}

/// A project rooted at `dir` with its `tasks` root created on disk — the
/// One slot per kind that can carry caller text, discovered the way a real
/// template's would be.
const KIND_SLOTS: &str = r#"#f(
  title: slot("title"),
  weight: slot("weight", kind: "int"),
  ratio: slot("ratio", kind: "float"),
  pick: slot("pick", kind: "choice", choices: ("a",)),
  tags: slot("tags", kind: "list"),
  body: slot("body", kind: "text"),
)"#;

/// Ordinary prose that happens to hold every Typst markup leader.
const HAZARDS: &str = "create-*star* _u_ $x$ #h [b] <a> >g @at";

/// [`HAZARDS`] as it must appear inside a content block.
const HAZARDS_ESCAPED: &str = r"create-\*star\* \_u\_ \$x\$ \#h \[b\] \<a> >g \@at";

/// Escaping is a per-kind decision, and getting it wrong is a bug in either
/// direction: an unescaped `text` slot lets Typst reinterpret the value, and an
/// escaped literal kind writes the backslashes into the value itself.
#[test]
fn markup_safe_escapes_text_slots_and_nothing_else() {
    let specs = mindtape::discover_slots(KIND_SLOTS);
    let mut values = BTreeMap::new();
    for spec in &specs {
        values.insert(spec.name.clone(), RawValue::Str(HAZARDS.to_owned()));
    }

    let out = markup_safe(&values, &specs);

    assert_eq!(
        out.get("body"),
        Some(&RawValue::Str(HAZARDS_ESCAPED.to_owned())),
        "a text slot splices into markup verbatim, so it must be escaped"
    );
    for name in ["title", "weight", "ratio", "pick", "tags"] {
        assert_eq!(
            out.get(name),
            Some(&RawValue::Str(HAZARDS.to_owned())),
            "{name} renders as a self-delimiting literal; escaping it would turn \
             the backslashes into data"
        );
    }
}

#[path = "add_tests_subprocess.rs"]
mod subprocess;
