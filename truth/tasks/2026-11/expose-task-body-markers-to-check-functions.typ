#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Expose task body metadata markers to check functions",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 2,
  tags: ("mt", "tooling", "tests", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a check gets the body markers as one explicit field.],
      enforced-practice: helps[a project check can read the markers its task body emits.],
      config-over-code: helps[the loader exposes a generic marker list, not a goals field.],
      cheap-adoption: neutral,
      typst-over-markdown: helps[a check reads metadata instead of parsing prose.],
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: helps[a warm cache and a cold eval expose the same markers.],
      docs-match-code: helps[the check-function page describes the marker input.],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 22),
  links: parent("rewire-goals-through-typst-markers.typ"),
)

== AS-IS

+ 🔴 A check with `field = "markers"` receives `none`: `field_value` reads `x-` slots and the canonical task, and neither carries the body markers.
+ Add `#metadata((marker: "example", value: 1)) <example>` to a task body. 🔴 The loader harvests it, then keeps only `mindtape.task`. The rest never reaches the check.
+ Run `mt check` twice. 🔴 A warm cache entry stores only the task marker, so even a loader that kept the markers would lose them on the second run.

== TO-BE

- 🟢 `Task` carries every non-task marker its body emits, in document order, each with its label and value.
- 🟢 `BodyMarker` derives `Task`'s traits and serde, and doesn't carry a source location. A check breach still lands on the task's `task.with(…)` line.
- 🟢 `field_value` gives `field = "markers"` the array directly from `Task.markers`. It is the one reserved check input, not a canonical field. Every other input is a canonical field or an `x-<ns>` slot. A task with no markers yields an empty array.
- 🟢 The loader cache stores the marker list, so a warm hit and a cold eval return the same markers.
- 🟢 `mt eval --json`, `mt ls --json`, `mt at` and `mt board` keep their current shape. The check input is the consumer.

```typ
#metadata((marker: "example", value: 1)) <example>
```

== Changes

- `crates/mindtape/src/model/mod.rs`: a `BodyMarker` type (`label`, `value`) and `Task.markers`. `BodyMarker` derives `Task`'s traits (`Debug`, `Clone`, `PartialEq`) plus serde, because the cache stores the list. `Task` itself has no serde derive and gains none.
- `crates/mindtape/src/import_map/mod.rs`: the exhaustive `Task` literal gets `markers: Vec::new()`. A canonical record carries no markers.
- `crates/mindtape/src/validate/candidate.rs`: the one `task_at` call site passes `&[]`, the candidate door's value until the next leaf harvests it.
- `crates/mindtape/src/loader/mod.rs`: `assemble_task(item, marker, markers, source)` and `task_at(..., markers, source)`. The cold path maps harvest markers to `BodyMarker` and keeps every marker except `mindtape.task`.
- `crates/mindtape/src/loader/cache.rs`: the marker list in `Entry` and `Cached`. Bump `FORMAT_VERSION`.
- `crates/mindtape/src/rules/typst.rs`, `crates/mindtape/src/rules/mod.rs`: `field_value` resolves the reserved `markers` field from `Task.markers` before the `x-` branch. The array holds `(label, value)` dicts in document order. An unlabelled marker has `label = none`. `CheckSpec.field`'s doc says `markers` yields an empty array, not `none`, and `schema/config.schema.json` regenerates from it.
- `crates/mindtape/src/loader/tests_cache.rs`, `crates/mindtape/src/rules/typst_tests.rs`: a marker in a heading, a marker through an imported helper, warm/cold equality and an empty/absent field.
- `crates/mindtape/src/test_support/mod.rs`, `crates/mindtape/src/import_map/mod.rs` and every other exhaustive `Task` literal: `markers: Vec::new()`. `grep -rn 'Task {' crates/` finds them, and the compiler reports each one that spreads no `..`. A literal spreading `..bare_task(..)` or `..Default::default()` doesn't change.
- `www/content/rules.typ`: the Typst-check section describes the body-marker input and names `markers` the one reserved input.
- `CHANGELOG.md`: an `Added` entry under `[Unreleased]`.

== Alternatives

- Store a goal-specific field. Rejected: it hardcodes one consumer and still leaves other body markers invisible.
- Re-harvest on every cache hit. Rejected: it defeats the cache and still needs the check input.
- Put markers on `CanonicalTask`. Rejected: it widens `ls`, `task`, `set-ls` and `eval` and forces four schema bumps for a check-only input.

== Out of scope

- The candidate door (`mt add`, `mt import`, `mt link`, `mt recur`). The next leaf owns it. This leaf passes `&[]` so the build stays green.
- The eval/listing projection. No schema change here.
- A goal field, a collector named `goals`, or any marker-name convention in Rust.
- Filter and sort support over markers.

== Dogfood

The DSL leaf reads these markers, and the cache test proves a flip and a later `mt check` see the same list.
