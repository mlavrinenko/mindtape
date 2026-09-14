#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Read the artifacts field into the task model",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt",),
  estimate: (files: 11),
  kind: "feat",
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on("record-the-artifacts-a-task-expects-to-touch.typ"),
)

== Summary

`record-the-artifacts-a-task-expects-to-touch.typ` lands the `artifacts` call shape in the prelude, but the Rust layer never reads it: `Task` has no field for it, a wrong-typed entry goes unreported, `mt ls --json` and `mt import` drop it, and nothing refuses a glob in `created`.


== Scope

Mechanical: follow the seams the `assignee` core-field commit (c34199e) touched. The typed field on `Task` in `crates/mindtape/src/model.rs`, tested in `model_tests.rs`; a wrong-typed entry reported by position (`artifacts[1].path`) in `crates/mindtape/src/marker.rs`; an empty default at each struct literal the field forces (`test_support.rs`, `filter_tests.rs`, `crates/mindtape-cli/src/render/field_tests.rs`); and the round trip through `canonical.rs`, `import_map.rs` and `emit.rs`, so `mt ls --json`, `mt import` and `mt add` all carry it.

In `crates/mindtape/src/validate.rs`, with a red test, a `created` path holding a glob metacharacter (`*`, `?` or `[`) is an error naming the task and the position, while `changed` accepts globs. No path is ever checked against the tree: a spec names files it will create, and a closed task names files a later refactor deleted.


== Dogfood

Not applicable: plumbing only. The field is dogfooded in `compare-declared-artifacts-against-the-working-tree.typ`.

