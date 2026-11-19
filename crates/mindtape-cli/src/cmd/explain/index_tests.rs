#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! Unit tests for the bare `mt explain` index: the shape blocks, the rule
//! grouping, the default-first lists and the JSON envelope.

use mindtape::parse_config;
use mindtape::test_support::write_file;

use mindtape::explain::InputShape;
use mindtape::{ActiveRule, RuleSection, Severity};

use super::render::{input_line, list_line, render};
use super::{Index, Label, bare_name, collect, default_first, document};

const MORE: &str = "more: mt explain <add|rules|tags|views|dicts|templates>";

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
         [[tasks.rules]]\nname = \"verb-title\"\nassert = 'has(title)'\nseverity = \"error\"\n\
         [[tasks.rules]]\nassert = 'has(title)'\n\
         [[tasks.rules]]\nname = \"goals\"\n\
         check = { typst = \"goals.typ\", function = \"check-goals\", field = \"x-goals\" }\n\
         [[tasks.body.content.rules]]\nname = \"as-is\"\npath = [\"AS-IS\"]\nassert = \"non-empty\"\n\
         [[commits.rules]]\nname = \"names-a-task\"\nassert = 'has(refs)'\n\
         [[on.flip]]\nname = \"review-stale-task-before-starting\"\nwhen = 'into(\"wip\")'\n\
         actions = [{ list = \"@wip\", message = \"already in flight\" }, \
         { show = \"hi\", message = \"and this one too\" }]\n\
         [[tasks.collectors]]\nname = \"changes\"\nreturns = [\"path\"]\n\
         query = { section = \"Changes\", pattern = \"`([^`]+)`\" }\n\
         [[on.add]]\nname = \"file-by-month\"\nwhen = '!has_tag(\"swept\")'\n\
         actions = [{ dir = \"tasks/{date:YYYY}-{date:MM}\" }]\n\
         [views.default]\nfilter = \"\"\n\
         [views.blocked]\nfilter = \"blocked\"\n\
         [dicts.source]\nvalues = [\"a\", \"b\"]\n",
    )
    .unwrap()
}

/// The index for `config`, read against an empty project directory.
fn index_of(config: &str) -> Index {
    let dir = tempfile::tempdir().unwrap();
    let config = parse_config(config).unwrap();
    collect(&config, &[], dir.path())
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
    let index = collect(&config, &[], dir.path());
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
    assert_eq!(
        index.rules[0].1,
        [
            (Label::Error, vec!["verb-title".to_owned()]),
            (Label::Warn, vec!["[1]".to_owned(), "goals".to_owned()]),
        ]
    );
    assert_eq!(
        index.rules[3].1,
        [(
            Label::Warn,
            vec!["review-stale-task-before-starting".to_owned()]
        )]
    );
}

/// An `ActiveRule` carrying only the fields `bare_name` reads.
fn rule(section: RuleSection, name: &str, run: Option<&str>, message: Option<&str>) -> ActiveRule {
    ActiveRule {
        section,
        name: name.to_owned(),
        severity: Severity::Error,
        message: message.map(str::to_owned),
        when: None,
        assertion: None,
        check: None,
        run: run.map(str::to_owned),
    }
}

/// A rule prints its name without the section path. An unnamed hook has no
/// name, so it prints what it runs, else its message, else its own label.
#[test]
fn bare_name_drops_the_section_and_an_action_slot() {
    let cases = [
        (
            RuleSection::TasksRules,
            "tasks.rules.verb-title",
            None,
            None,
            "verb-title",
        ),
        (RuleSection::TasksRules, "tasks.rules[2]", None, None, "[2]"),
        (
            RuleSection::OnFlip,
            "on.flip.review-stale-task-before-starting.actions[1]",
            None,
            None,
            "review-stale-task-before-starting",
        ),
        (
            RuleSection::OnFlip,
            "on.flip[0].actions[0]",
            Some("scripts/test"),
            None,
            "scripts/test",
        ),
        (
            RuleSection::OnFlip,
            "on.flip[0].actions[0]",
            None,
            Some("not yet"),
            "not yet",
        ),
        (
            RuleSection::OnFlip,
            "on.flip[0].actions[0]",
            None,
            None,
            "on.flip[0]",
        ),
    ];
    for (section, name, run, message, expected) in cases {
        let rule = rule(section, name, run, message);
        assert_eq!(bare_name(section, &rule), expected, "{name}");
    }
}

#[test]
fn lists_derived_fields_without_the_prefix() {
    let (dir, config) = project();
    assert_eq!(collect(&config, &[], dir.path()).derived, ["changes"]);
}

#[test]
fn puts_default_first_and_keeps_the_rest_in_order() {
    let (dir, config) = project();
    let index = collect(&config, &[], dir.path());
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
/// `terminal`, and the actors with their kind and roles. The `task:` block
/// carries the scales, the vocabularies, the extension with the rule that reads
/// it, the templates with the default, and the rules that check a task. The
/// `commits:`, `flip:` and `rm:` blocks carry their own.
#[test]
fn renders_the_shape_blocks_before_the_listings() {
    let (dir, config) = project();
    let text = render(&collect(&config, &[], dir.path()), 100);
    let expected = "project:
  roots: tasks (tag: tasks)
  statuses:
    open: proposed, wip
    terminal: done
  actors: max (human, maintainer), agent (llm)
task:
  priority: ice (confidence 0-1, ease 1-10, impact 1-10) -> 0-9
  difficulty: 0-9
  estimate: src-files (optional)
  kinds: feat, fix
  links: depends-on, parent, recurred-from, supersedes, duplicates, related
  comments: forbidden
  extensions: x-goals (goals)
  templates: decision, task (default: task)
  rules (error): verb-title
  rules (warn): [1], goals
  body.content (error): as-is
commits:
  trailer: Refs
  rules (warn): names-a-task
add:
  hooks: file-by-month
flip:
  hooks (warn): review-stale-task-before-starting
rm:
  filter: terminal
";
    assert!(text.starts_with(expected), "{text}");
}

/// A project that constrains every key prints no `absent:` line, because the
/// line has nothing to list. The JSON keeps the key as an empty array.
#[test]
fn a_project_that_constrains_every_key_prints_no_absent_line() {
    let (dir, config) = project();
    let index = collect(&config, &[], dir.path());
    assert!(index.shape.absent.is_empty());
    let text = render(&index, 100);
    assert!(!text.contains("absent"), "{text}");
    let value = serde_json::to_value(document(&index)).unwrap();
    assert_eq!(value["absent"], serde_json::json!([]));
    assert_eq!(value["waits"], serde_json::json!([]));
    assert_eq!(value["add"], serde_json::json!(["file-by-month"]));
}

/// A key the project leaves open prints what its absence permits: `any` for a
/// free vocabulary, `none` for an empty registry, and the policy word for
/// comments. The JSON keeps the explicit list under `absent`.
#[test]
fn a_project_that_leaves_keys_open_prints_what_each_absence_permits() {
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\nstatus = []\n").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let index = collect(&config, &[], dir.path());
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
    for line in [
        "  statuses: any\n",
        "  actors: none\n",
        "  estimate: any\n",
        "  kinds: any\n",
        "  comments: allowed\n",
        "  extensions: none\n",
    ] {
        assert!(text.contains(line), "missing {line:?} in:\n{text}");
    }
    assert!(!text.contains("absent"), "{text}");
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
    let extensions = collect(&config, &[], dir.path()).shape.task.extensions;
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
    let text = render(&collect(&config, &[], dir.path()), 100);
    for line in [
        "  priority: flat\n",
        "  estimate: days (required: !terminal)\n",
        "  comments: warned\n",
        "  extensions: x-note\n",
    ] {
        assert!(text.contains(line), "missing {line:?} in:\n{text}");
    }

    let required = parse_config("[tasks.estimate]\nunits = [\"days\"]\nrequired = true\n").unwrap();
    let text = render(&collect(&required, &[], dir.path()), 100);
    assert!(text.contains("  estimate: days (required)\n"), "{text}");
}

/// An unnamed add hook prints the directory it files into, and its
/// `on.add[<index>]` label when it files nowhere.
#[test]
fn an_unnamed_add_hook_prints_the_directory_it_files_into() {
    let index = index_of(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [[on.add]]\nwhen = '!has_tag(\"swept\")'\n\
         actions = [{ dir = \"tasks/swept\" }]\n",
    );
    assert_eq!(index.add, ["tasks/swept"]);
    let text = render(&index, 100);
    assert!(text.contains("add:\n  hooks: tasks/swept\n"), "{text}");

    let index = index_of(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [[on.add]]\nwhen = ''\n\
         actions = [{ set = { kind = \"fix\" } }]\n",
    );
    assert_eq!(index.add, ["on.add[0]"]);
}

/// A framework's inputs print what each accepts: a number its own range, a
/// bool its name, and an enum its values behind a pipe. The score's scale
/// follows the arrow.
#[test]
fn prints_each_framework_input_with_what_it_accepts() {
    let dir = tempfile::tempdir().unwrap();
    for (framework, expected) in [
        (
            "ice",
            "  priority: ice (confidence 0-1, ease 1-10, impact 1-10) -> 0-9\n",
        ),
        (
            "eisenhower",
            "  priority: eisenhower (important, urgent) -> 0-9\n",
        ),
        (
            "moscow",
            "  priority: moscow (level: must|should|could|wont) -> 0-9\n",
        ),
    ] {
        let config =
            parse_config(&format!("[tasks.priority]\nframework = \"{framework}\"\n")).unwrap();
        let text = render(&collect(&config, &[], dir.path()), 100);
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
}

/// A number input with no bounds prints its name alone, and so does a kind the
/// renderer does not know.
#[test]
fn prints_an_input_with_no_bounds_by_its_name() {
    let mut input = InputShape {
        name: "impact".to_owned(),
        kind: "number".to_owned(),
        min: None,
        max: None,
        values: Vec::new(),
    };
    assert_eq!(input_line(&input), "impact");
    input.kind = "text".to_owned();
    assert_eq!(input_line(&input), "impact");
}

/// A command the user has not allowed prints under `not allowed` instead of
/// its severity, and the `allow:` line says how to record the approval. The
/// JSON carries the same label.
#[test]
fn a_command_the_user_has_not_allowed_prints_under_not_allowed() {
    let config = parse_config(
        "[[tasks.waits]]\nname = \"locked\"\nrun = \"scripts/locked {id}\"\n\
         [[on.flip]]\nwhen = 'into(\"done\")'\n\
         actions = [{ run = \"scripts/test\", gate = true }]\n",
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let index = collect(&config, &[], dir.path());
    let text = render(&index, 100);
    for line in [
        "  waits (not allowed): scripts/locked {id}\n",
        "flip:\n  hooks (not allowed): scripts/test\n",
    ] {
        assert!(text.contains(line), "missing {line:?} in:\n{text}");
    }
    assert!(!text.contains("refusals:"), "{text}");
    assert!(
        text.ends_with(
            "allow: mt allow records the approval for each command marked not allowed\n"
        ),
        "{text}"
    );

    let value = serde_json::to_value(document(&index)).unwrap();
    assert_eq!(value["waits"], serde_json::json!(["scripts/locked {id}"]));
    assert_eq!(
        value["rules"]["on.flip"],
        serde_json::json!({"not allowed": ["scripts/test"]})
    );
}

#[test]
fn wraps_after_a_comma_when_the_next_entry_passes_the_width() {
    let items = ["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()];
    assert_eq!(list_line("tags", &items, 18), "tags: alpha, beta,\n  gamma");
    assert_eq!(list_line("tags", &[], 18), "tags: none");
}

#[test]
fn the_index_ends_on_the_listings_that_go_deeper() {
    let (dir, config) = project();
    let text = render(&collect(&config, &[], dir.path()), 100);
    assert!(text.ends_with(&format!("\n{MORE}\n")), "{text}");
}

#[test]
fn json_carries_the_same_shape_and_grouping() {
    let (dir, config) = project();
    let value = serde_json::to_value(document(&collect(&config, &[], dir.path()))).unwrap();
    mindtape::test_support::assert_schema_at(&value, "explain", 5);
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
                "inputs": [
                    {"name": "confidence", "kind": "number", "min": 0.0, "max": 1.0, "values": []},
                    {"name": "ease", "kind": "number", "min": 1.0, "max": 10.0, "values": []},
                    {"name": "impact", "kind": "number", "min": 1.0, "max": 10.0, "values": []},
                ],
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
    assert_eq!(value["waits"], serde_json::json!([]));
    assert_eq!(value["add"], serde_json::json!(["file-by-month"]));
    assert_eq!(
        value["rules"],
        serde_json::json!({
            "tasks.rules": {"error": ["verb-title"], "warn": ["[1]", "goals"]},
            "tasks.body.content.rules": {"error": ["as-is"]},
            "commits.rules": {"warn": ["names-a-task"]},
            "on.flip": {"warn": ["review-stale-task-before-starting"]},
        })
    );
    assert_eq!(value["derived"], serde_json::json!(["changes"]));
}
