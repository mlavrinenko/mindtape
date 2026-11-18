#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! Unit tests for the bare `mt explain` index: the shape blocks, the rule
//! grouping, the default-first lists and the JSON envelope.

use mindtape::parse_config;
use mindtape::test_support::write_file;

use mindtape::RuleSection;

use super::{bare_name, collect, default_first, document, list_line, render};

const NEXT: &str = "next: mt explain [<add|rules|tags|views|dicts>] [--json]";

/// A config with every shape block populated. It declares a three-value
/// legend, two actors, an estimate unit, a forbidden comment policy, an
/// `x-goals` rule and a priority framework. It also declares the rules, tags,
/// views and dictionaries the listings read.
fn config() -> mindtape::Config {
    parse_config(
        "[tasks.tags]\nallowed = [\"cli\", \"default\", \"mt\"]\n\
         [[tasks.status]]\nvalue = \"proposed\"\n\
         [[tasks.status]]\nvalue = \"wip\"\n\
         [[tasks.status]]\nvalue = \"done\"\nterminal = true\n\
         [[roles]]\nname = \"maintainer\"\n\
         [[actors]]\nname = \"max\"\nkind = \"human\"\nroles = [\"maintainer\"]\n\
         [[actors]]\nname = \"agent\"\nkind = \"llm\"\n\
         [tasks.priority]\nframework = \"ice\"\n\
         [tasks.estimate]\nunits = [\"src-files\"]\n\
         [tasks.comments]\npolicy = \"forbid\"\n\
         [tasks.fields.kind]\nvalues = [\"feat\", \"fix\"]\n\
         [[tasks.rules]]\nname = \"verb-title\"\nassert = 'has(title)'\n\
         [[tasks.rules]]\nassert = 'has(title)'\n\
         [[tasks.rules]]\nname = \"goals\"\n\
         check = { typst = \"goals.typ\", function = \"check-goals\", field = \"x-goals\" }\n\
         [[tasks.body.content.rules]]\nname = \"as-is\"\npath = [\"AS-IS\"]\nassert = \"non-empty\"\n\
         [[commits.rules]]\nname = \"names-a-task\"\nassert = 'has(refs)'\n\
         [[on.flip]]\nname = \"review-stale-task-before-starting\"\nwhen = 'into(\"wip\")'\n\
         actions = [{ list = \"@wip\", message = \"already in flight\" }, \
         { run = \"echo hi\", message = \"and this one too\" }]\n\
         [[tasks.collectors]]\nname = \"changes\"\nreturns = [\"path\"]\n\
         query = { section = \"Changes\", pattern = \"`([^`]+)`\" }\n\
         [views.default]\nfilter = \"\"\n\
         [views.blocked]\nfilter = \"blocked\"\n\
         [dicts.source]\nvalues = [\"a\", \"b\"]\n",
    )
    .unwrap()
}

/// [`config`] with a root holding one template beside the built-in `task`.
fn project() -> (tempfile::TempDir, mindtape::Config) {
    let dir = tempfile::tempdir().unwrap();
    write_file(dir.path(), ".mindtape/templates/decision.typ", "");
    (dir, config())
}

#[test]
fn groups_rules_by_section_and_drops_a_section_with_none() {
    let (dir, config) = project();
    let index = collect(&config, dir.path());
    let sections: Vec<&str> = index.rules.iter().map(|(s, _)| s.path()).collect();
    assert_eq!(
        sections,
        [
            "tasks.rules",
            "tasks.body.content.rules",
            "commits.rules",
            "on.flip"
        ]
    );
    assert_eq!(index.rules[0].1, ["verb-title", "[1]", "goals"]);
    assert_eq!(index.rules[3].1, ["review-stale-task-before-starting"]);
}

#[test]
fn bare_name_drops_the_section_and_an_action_slot() {
    assert_eq!(
        bare_name(RuleSection::TasksRules, "tasks.rules.verb-title"),
        "verb-title"
    );
    assert_eq!(bare_name(RuleSection::TasksRules, "tasks.rules[2]"), "[2]");
    assert_eq!(
        bare_name(
            RuleSection::OnFlip,
            "on.flip.review-stale-task-before-starting.actions[1]"
        ),
        "review-stale-task-before-starting"
    );
    assert_eq!(
        bare_name(RuleSection::OnFlip, "on.flip[0].actions[0]"),
        "[0]"
    );
}

#[test]
fn lists_derived_fields_without_the_prefix() {
    let (dir, config) = project();
    assert_eq!(collect(&config, dir.path()).derived, ["changes"]);
}

#[test]
fn puts_default_first_and_keeps_the_rest_in_order() {
    let (dir, config) = project();
    let index = collect(&config, dir.path());
    assert_eq!(index.tags, ["default", "cli", "mt"]);
    assert_eq!(index.views, ["default", "blocked"]);
    assert_eq!(index.dicts[0].0, "mindtape-dev-verbs");
    assert!(index.dicts[0].1 > 0);
    assert_eq!(index.dicts[1].0, "mindtape-verbs");
    assert_eq!(index.dicts[2], ("source".to_owned(), 2));
    assert_eq!(
        default_first(vec!["b", "default", "a"], |s| s),
        ["default", "b", "a"]
    );
}

/// The shape blocks lead the index, and every line is a config value. The
/// `project:` block carries the roots with their tags, the legend split by
/// `terminal`, the actors with their kind and roles, and the `rm` filter. The
/// `task:` block carries the scales, the vocabularies, the extension with the
/// rule that reads it, and the templates with the default.
#[test]
fn renders_the_shape_blocks_before_the_listings() {
    let (dir, config) = project();
    let text = render(&collect(&config, dir.path()), 100);
    let expected = "project:
  roots: tasks (tag: tasks)
  statuses:
    open: proposed, wip
    terminal: done
  actors: max (human, maintainer), agent (llm)
  rm: terminal only
task:
  priority: ice (confidence, ease, impact)
  difficulty: 0-9
  estimate: src-files (optional)
  kinds: feat, fix
  links: depends-on, parent, recurred-from, supersedes, duplicates, related
  comments: forbidden
  extensions: x-goals (goals)
  templates: decision, task (default: task)
commits:
  trailer: Refs
absent: none
rules:
";
    assert!(text.starts_with(expected), "{text}");
}

/// A key the project leaves open stays out of its block, and `absent` names it
/// once. The reader then sees what the defaults fill in.
#[test]
fn a_project_that_leaves_keys_open_names_them_in_absent() {
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\nstatus = []\n").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let index = collect(&config, dir.path());
    assert_eq!(
        index.shape.absent,
        [
            "statuses",
            "actors",
            "estimate",
            "kinds",
            "comments",
            "extensions"
        ]
    );
    let text = render(&index, 100);
    assert!(
        text.contains("absent: statuses, actors, estimate, kinds, comments, extensions\n"),
        "{text}"
    );
    for key in [
        "  statuses:",
        "  actors:",
        "  estimate:",
        "  kinds:",
        "  extensions:",
    ] {
        assert!(!text.contains(key), "{key} in:\n{text}");
    }
}

/// An `x-` field a `check` rule reads is an extension even when
/// `[tasks.fields]` never declares it, and an unnamed rule reports by its
/// position.
#[test]
fn extensions_name_the_x_fields_and_the_rules_that_read_them() {
    let config = parse_config(
        "[tasks.fields.x-speconaut]\nvalues = [\"spec\"]\n\
         [[tasks.rules]]\ncheck = { typst = \"a.typ\", function = \"f\", field = \"x-goals\" }\n\
         [[tasks.rules]]\nname = \"reads-spec\"\n\
         check = { typst = \"a.typ\", function = \"f\", field = \"x-speconaut\" }\n",
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let extensions = collect(&config, dir.path()).shape.task.extensions;
    let names: Vec<(&str, Vec<&str>)> = extensions
        .iter()
        .map(|extension| {
            (
                extension.name.as_str(),
                extension.rules.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    assert_eq!(
        names,
        [
            ("x-goals", vec!["[0]"]),
            ("x-speconaut", vec!["reads-spec"])
        ]
    );
}

/// A framework with no inputs prints its name alone, and the estimate's
/// `required` and the comment policy each print the word their config value
/// means. An `x-` field no rule reads prints without a parenthetical.
#[test]
fn prints_a_framework_with_no_inputs_and_the_open_forms() {
    let config = parse_config(
        "[tasks.priority]\nframework = \"flat\"\n\
         [tasks.priority.frameworks.flat]\nscore = \"5\"\n\
         [tasks.estimate]\nunits = [\"days\"]\nrequired = \"!terminal\"\n\
         [tasks.comments]\npolicy = \"warn\"\n\
         [tasks.fields.x-note]\nvalues = [\"a\"]\n",
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let text = render(&collect(&config, dir.path()), 100);
    for line in [
        "  priority: flat\n",
        "  estimate: days (required: !terminal)\n",
        "  comments: warned\n",
        "  extensions: x-note\n",
    ] {
        assert!(text.contains(line), "missing {line:?} in:\n{text}");
    }

    let required = parse_config("[tasks.estimate]\nunits = [\"days\"]\nrequired = true\n").unwrap();
    let text = render(&collect(&required, dir.path()), 100);
    assert!(text.contains("  estimate: days (required)\n"), "{text}");
}

#[test]
fn wraps_after_a_comma_when_the_next_entry_passes_the_width() {
    let items = ["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()];
    assert_eq!(
        list_line("tags", &items, 18, "  "),
        "tags: alpha, beta,\n  gamma"
    );
    assert_eq!(list_line("tags", &[], 18, "  "), "tags: none");
}

#[test]
fn the_index_ends_on_the_command_to_run_next() {
    let (dir, config) = project();
    let text = render(&collect(&config, dir.path()), 100);
    assert!(text.ends_with(&format!("\n{NEXT}\n")), "{text}");
}

#[test]
fn json_carries_the_same_shape_and_grouping() {
    let (dir, config) = project();
    let value = serde_json::to_value(document(&collect(&config, dir.path()))).unwrap();
    mindtape::test_support::assert_schema_at(&value, "explain", 2);
    assert_eq!(
        value["project"],
        serde_json::json!({
            "roots": [{"path": "tasks", "tag": "tasks"}],
            "statuses": {"open": ["proposed", "wip"], "terminal": ["done"]},
            "actors": [
                {"name": "max", "kind": "human", "roles": ["maintainer"]},
                {"name": "agent", "kind": "llm", "roles": []},
            ],
            "rm": "terminal",
        })
    );
    assert_eq!(
        value["task"],
        serde_json::json!({
            "priority": {
                "framework": "ice",
                "inputs": ["confidence", "ease", "impact"],
                "min": 0.0,
                "max": 9.0,
            },
            "difficulty": {"min": 0.0, "max": 9.0},
            "estimate": {"units": ["src-files"], "required": "false"},
            "kinds": ["feat", "fix"],
            "links": [
                "depends-on", "parent", "recurred-from", "supersedes", "duplicates", "related",
            ],
            "comments": "forbidden",
            "extensions": [{"name": "x-goals", "rules": ["goals"]}],
            "templates": {"names": ["decision", "task"], "default": "task"},
        })
    );
    assert_eq!(value["commits"], serde_json::json!({"trailer": "Refs"}));
    assert_eq!(value["absent"], serde_json::json!([]));
    assert_eq!(
        value["rules"],
        serde_json::json!({
            "tasks.rules": ["verb-title", "[1]", "goals"],
            "tasks.body.content.rules": ["as-is"],
            "commits.rules": ["names-a-task"],
            "on.flip": ["review-stale-task-before-starting"],
        })
    );
    assert_eq!(value["derived"], serde_json::json!(["changes"]));
}
