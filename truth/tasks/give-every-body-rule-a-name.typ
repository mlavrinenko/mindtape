#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Give every body rule a name",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt", "gating"),
  estimate: (files: 5),
  kind: "feat",
  assignee: "agent",
  status: wip(2026, 9, 16),
)

== Problem

+ Add `name = "problem-steps"` to any `[[tasks.body.content.rules]]` entry in `.mindtape/config.toml`.
+ Run `mt check`.
+ 🔴 The config fails to load on the unknown field `name`. Only `[[tasks.rules]]` entries carry one (`crates/mindtape/src/config_rules.rs:59`), so nothing can point at a single section rule to switch it off or change its threshold.


== Goals

#goals(
  product: (
    readable-tasks: neutral,
    implementable-tasks: neutral,
    enforced-practice: helps[a named rule can be switched off alone instead of dropping a whole pack],
    cheap-adoption: helps[rule-by-rule overrides are what make a one-line `extends` safe to adopt],
    typst-over-markdown: neutral,
    plain-files: neutral,
  ),
  process: (
    sound-codebase: neutral,
    docs-match-code: neutral,
  ),
)

== TO-BE

- 🟢 Every `[[tasks.body.headers.rules]]` and `[[tasks.body.content.rules]]` entry accepts an optional `name`, and `mt check` prints it beside a violation.
- 🟢 Two rules sharing a name in one config is a config error naming both.

```toml
[[tasks.body.content.rules]]
name = "problem-steps"
path = ["Problem"]
assert = "non-empty"
```


== Changes

- `crates/mindtape/src/body_headers.rs`: an optional `name`, carried into the violation.
- `crates/mindtape/src/body_content.rs`: the same.
- `crates/mindtape/src/config_tests_headers.rs` and `crates/mindtape/src/body_content_tests.rs`: red tests first, the Problem steps among them.
- `schema/config.schema.json`: regenerate.

== Alternatives

- Address a rule by its position, `tasks.body.content.rules[3]`: reordering the config would silently retarget every `disable`.
- Derive an id from a rule's content: tweaking one threshold changes the id, so an override could never find its target.


== Out of scope

- Anything that uses the names: `extends` and `disable` are `let-a-config-extend-rule-packs.typ`.
- `[[tasks.rules]]`: its `name` already exists.


== Dogfood

Built in: this repo's section rules get names when they move into the pack in `ship-the-task-practice-as-the-mindtape-tasks-pack.typ`.

