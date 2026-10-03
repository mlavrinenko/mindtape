#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Reject unknown keys under tasks.priority",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 3),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a misspelled priority key fails the load instead of doing nothing],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the schema stops allowing keys the loader ignores],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== Problem

+ resolving config keys named in doc prose against schema/config.schema.json
+ 🔴 #raw("crates/mindtape/src/config_tasks.rs"): PriorityConfig flattens NumericField with serde(flatten), which cannot combine with deny\_unknown\_fields, so an unknown key under \[tasks.priority\] loads without an error.

In a copy of `examples/minimal`, `bogus-key = 1` under `[tasks.estimate]` fails the load and the same line under `[tasks.priority]` does not:

```
Error: load <copy>/.mindtape/config.toml
    0: parse error: unknown field `bogus-key`, expected `units` or `required`
       in `tasks.estimate`
```

With `[tasks.priority]` the run reports only the fixture's lib path errors, and no parse error. `schema/config.schema.json` gives `PriorityConfig` no `additionalProperties: false`; every other table there has it.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/config_tasks.rs").

== TO-BE

- 🟢 An unknown key under `[tasks.priority]` fails the load and names the key, as it does under `[tasks.estimate]`.
- 🟢 `schema/config.schema.json` sets `additionalProperties: false` on `PriorityConfig`.


== Changes

- `crates/mindtape/src/config_tasks.rs`: deserialize `PriorityConfig` without `serde(flatten)`, or check its leftover keys after the flattened read.
- `schema/config.schema.json`: regenerated.
- A test that loads `[tasks.priority]` with an unknown key and expects the error.


== Rollback

Revert the commit.

== Out of scope

- Other flattened config structs: none exist today.


== Dogfood

This repo's own `[tasks.priority]` table is checked by every `mt` run.
