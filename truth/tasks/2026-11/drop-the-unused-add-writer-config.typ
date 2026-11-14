#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Drop the unused add-writer config argument",
  kind: "dev",
  priority: ice(
    impact: 2,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  links: (
    (
      task: "let-rules-and-templates-choose-where-new-tasks-land.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "tooling", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  status: done(2026, 11, 12),
  assignee: "agent",
)

== AS-IS

+ Reviewing the on-add branch after `just check`.
+ 🔴 #raw("crates/mindtape/src/model/writable.rs"): `apply_op` takes a `Config` it never reads, and the test-only token helpers need a dead-code allow.

```rust
pub fn apply_op(task: &mut Task, field: &WritableField, op: &FieldOp, _config: &Config)

#[allow(dead_code)]
pub fn append_list_value(...)
```


== TO-BE

- 🟢 `apply_op` takes only the task, field and operation.
- 🟢 Helpers only tests use are `cfg(test)` and keep no dead-code allow.


== Changes

- `crates/mindtape/src/model/writable.rs`: drop the unused parameter.
- `crates/mindtape-cli/src/cmd/add/hooks.rs`, `draft.rs`, `pipeline.rs`: update the callers.
- `crates/mindtape/src/model/writable_tests.rs`: update the writer tests.
- `crates/mindtape-cli/src/cmd/add/tokens.rs`: gate the map-shaped test helpers.


== Out of scope

- The writer's public field table and literal renderer.


== Dogfood

The shared writer advertises only inputs it uses.

