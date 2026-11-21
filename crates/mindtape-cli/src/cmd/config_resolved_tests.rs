#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

//! Tests for `mt config show --resolved`, the view that prints every default.
//! `config_tests.rs` covers the collected view and the origin listing. This
//! file covers the resolved one, whose whole point is that its output reads
//! back as the same config.

use mindtape::test_support::tempdir_with_config;
use mindtape::{load_config, parse_config};

use super::render_effective;

/// A config exercising each shape whose TOML form is not the plain derived
/// one. That covers a `Requirement` as `true` and as a filter, `IdFormat`,
/// the flattened `PriorityConfig`, a heading path's string and table steps,
/// and every `ContentAssert` form.
///
/// It also declares the three collector queries with their three `returns`
/// forms, a `DedupMode`, both `ConfigCommand` shapes, and the untagged
/// `RootSpec`, `DeriveTags`, `SectionSpec` and `AddValue` enums. Each of those
/// hand-writes its `Deserialize`, so a derived `Serialize` cannot mirror it on
/// its own. The round trip below checks each one.
const RICH_CONFIG: &str = r#"
[project]
code = "rich"

[add]
interactive = "never"
comments = false
verify-when = "!tty()"

[format]
enabled = false

[tasks.id]
format = "uuid-v4"

[tasks.naming]
stem = "{date}-{slug}"
dedup = "skip"

[[tasks.roots]]
path = "backlog"
name = "backlog"
derive-tags = ["sub/**"]
exclude = ["archive/**"]

[tasks.derive-tags]
enabled = true
collect = ["breadcrumbs"]
separator = "/"
slugify = false
breadcrumb-root = true

[tasks.priority]
framework = "ice"
allow-manual = false
required = "!terminal"
example = 5.0

[[tasks.difficulty.anchors]]
value = 0.0
label = "easy"

[[tasks.difficulty.anchors]]
value = 0.75
label = "hard"
glyph = "!"
color = "red"

[tasks.fields.kind]
values = ["bug"]
default = "bug"
required = true
example = "bug"

[tasks.assignee]
allowed = ["agent"]
required = 'status == "wip"'
example = "agent"

[tasks.estimate]
units = ["hours"]
required = false
example = { hours = 2.0 }

[tasks.tags]
allowed = ["mt"]

[tasks.links]
kinds = [
  { name = "vendor", max = 2, note = "required", validate = "regex", regex = "^vendor:" },
]

[tasks.recur]
enabled = false

[tasks.comments]
policy = "warn"
allow-pattern = "^TODO"

[[tasks.waits]]
name = "locked"
run = ["bash", "-c", "exit 0"]

[[tasks.collectors]]
name = "word-count"
query = { section = "Changes", pattern = "(\\w+)", group = 1 }
returns = ["string"]

[[tasks.collectors]]
name = "module"
query = { typst = "tools.typ", function = "module", field = "body" }
returns = { module = "string", names = ["string"] }

[[tasks.collectors]]
name = "todo"
query = { ast_grep = { language = "rust", pattern = "todo!()", glob = "**/*.rs" } }
returns = ["path"]

[tasks.body.headers]
rules = [
  { action = "require", path = ["AS-IS"] },
  { name = "one-todo", action = "allowlist", scope = [{ match = "^Tasks$", recursive = true }], allowed = ["AS-IS", "TO-BE"], when = "true", message = "keep it short" },
]

[tasks.body.content]
rules = [
  { path = ["AS-IS"], assert = "non-empty" },
  { path = ["Changes"], assert = { words = { min = 3 } }, message = "write more" },
  { path = ["TO-BE"], assert = { words = { max = 900 } } },
  { path = ["Changes"], assert = { must-match = "^-" } },
  { path = ["AS-IS"], assert = { must-not-match = "TODO" } },
  { path = ["TO-BE"], assert = { density = { pattern = "!", max = 1, per = "paragraph" } } },
]

[[tasks.rules]]
name = "no-full-stop"
assert = '!matches(title, "\\.$")'
severity = "warn"

[[tasks.rules]]
name = "typed"
when = "status == \"wip\""
assert = "has(title)"
message = "write a title"

[commits]
trailer = "Refs"
external-refs = ["Fixes"]
run = "bash scripts/check.sh"

[[commits.rules]]
name = "prose"
when = "in_range"
assert = '!matches(subject, "^\\.$")'
message = "drop the full stop"

[[on.flip]]
name = "review"
when = 'into("done")'
actions = [
  { show = "done" },
  { run = "bash scripts/after.sh", gate = true, message = "fix it" },
]

[[on.add]]
name = "stamp"
when = "template == \"task\""
actions = [
  { set = { kind = "bug", priority = 3 }, append = { tags = ["mt"] } },
  { dir = "backlog" },
]

[[on.rm]]
when = "true"
actions = [{ run = ["bash", "-c", "exit 0"] }]

[[actors]]
name = "agent"
kind = "llm"
roles = ["maintainer"]

[[roles]]
name = "maintainer"

[[typst.local-packages]]
name = "extra"
path = "lib/extra"

[views.default]
filter = 'status == "wip"'
sort = ["priority:desc"]
columns = ["title", "status"]
description = "work in progress"

[dicts.verbs]
values = ["add", "wire"]
include = ["mindtape-verbs"]
case = "exact"

[ls]
default-view = "default"
default-sort = ["priority:desc"]
date-format = "YYYY-MM-DD"
overdue-color = "red"
hyperlink-target = "file://{abs}"

[[ls.rules]]
name = "main"
condition = "on_branch(\"main\")"
import-views = ["default"]
"#;

#[test]
fn a_rule_with_no_severity_prints_the_level_it_reports_at() {
    let dir = tempdir_with_config(
        "[[tasks.rules]]\nname = \"no-full-stop\"\nassert = '!matches(title, \"\\\\.$\")'\n",
    );
    // A top-level `severity` line belongs to a rule: the floor's
    // `[templates] fallback-severity` starts with `fallback-`.
    let levels = |text: &str| {
        text.lines()
            .filter(|line| line.starts_with("severity = "))
            .count()
    };

    let collected = render_effective(dir.path(), false, false).unwrap();
    assert_eq!(levels(&collected), 0, "{collected}");

    let resolved = render_effective(dir.path(), false, true).unwrap();
    assert_eq!(levels(&resolved), 1, "{resolved}");
}

#[test]
fn the_resolved_output_prints_the_sections_a_layer_left_unset() {
    // `examples/minimal` sets none of these five, so the collected view drops
    // every one of them.
    let dir = tempdir_with_config("[project]\ncode = \"demo\"\n");
    let text = render_effective(dir.path(), false, true).unwrap();
    for section in [
        "[tasks.links]",
        "[tasks.estimate]",
        "[tasks.comments]",
        "[add]",
        "[format]",
    ] {
        assert!(text.contains(section), "{section} missing from {text}");
    }
}

/// A project holding [`RICH_CONFIG`], plus the module its `typst` collector
/// names, which config load requires to exist relative to the root.
fn rich_project() -> tempfile::TempDir {
    let dir = tempdir_with_config(RICH_CONFIG);
    std::fs::write(
        dir.path().join("tools.typ"),
        "#let module(body) = (module: \"core\", names: (\"a\",))\n",
    )
    .unwrap();
    dir
}

#[test]
fn the_resolved_output_round_trips_as_a_config() {
    let dir = rich_project();

    let text = render_effective(dir.path(), false, true).unwrap();
    let reparsed = parse_config(&text).unwrap();
    assert_eq!(reparsed, load_config(dir.path()).unwrap());
}

#[test]
fn the_resolved_json_mirrors_the_resolved_toml() {
    let dir = rich_project();

    let toml_text = render_effective(dir.path(), false, true).unwrap();
    let json_text = render_effective(dir.path(), true, true).unwrap();

    let from_toml = parse_config(&toml_text).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_text).unwrap();
    assert_eq!(from_toml, super::tests::config_from_json(&json_value));
}

#[test]
fn the_resolved_output_omits_the_layer_only_keys() {
    let dir = tempdir_with_config(
        "extends = [\"mindtape:tasks\"]\ndisable = [\"start-title-with-verb\"]\n",
    );

    let text = render_effective(dir.path(), false, true).unwrap();
    assert!(!text.contains("extends"), "{text}");
    assert!(!text.contains("disable"), "{text}");

    let reparsed = parse_config(&text).unwrap();
    assert!(reparsed.extends.is_empty());
    assert!(reparsed.disable.is_empty());
    assert!(
        reparsed
            .tasks
            .rules
            .iter()
            .all(|rule| rule.name.as_deref() != Some("start-title-with-verb")),
        "a disabled rule stays out, rather than printing and re-enabling"
    );
}
