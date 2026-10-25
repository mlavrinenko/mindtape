#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep a count estimate's source form through mt ls --json and mt import",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (
    src-files: count(1, grep(
      `canonical\.estimate|fn emit_estimate|estimate: task\.estimate\.clone\(\)|fn task_json`,
      "crates/**",
    )),
  ),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a task moved through `mt import` keeps the queries its cap rule reads, so the cap still tracks the code],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[a script that rewrites tasks as JSON gets back the `count(...)` the author wrote],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 25),
)

== Problem

+ In a git project whose `[tasks.estimate.queries]` has `src-files = "crates/**"`, with two files under `crates/` naming `PathFilter`, file a task with `estimate: (src-files: count(2, grep("PathFilter", "crates/**")))`.
+ Run `mt ls --json > /tmp/export.json`, delete the task file, and run `mt import /tmp/export.json`.
+ 🔴 The written task says `estimate: (src-files: 4),`: the queries are gone, and the number stays at what they matched on the day of the export.

== TO-BE

- 🟢 The canonical JSON carries each `count(...)` unit under `estimate_counts`, as `{count, queries, problem}`, beside its resolved number in `estimate`, in `mt ls --json` and `mt eval --json` alike.
- 🟢 `mt import` writes such a unit back as the `count(...)` it came from, so `mt ls --json | mt import` round-trips it.


== Changes

- `crates/mindtape-cli/tests/it/estimate_queries_e2e.rs`: the red test first, an export and import that gives back the `count(...)`.
- `crates/mindtape/src/model/canonical.rs`: `estimate_counts` on `CanonicalTask`, read and written.
- `crates/mindtape/src/import_map/mod.rs`: map `estimate_counts` onto the task.
- `crates/mindtape/src/emit/mod.rs`: emit a count unit as `count(n, grep(...), glob(...))`.
- `crates/mindtape-cli/src/cmd/eval.rs`: drop its own `estimate_counts` insertion, which the canonical task carries.


== Alternatives

- Write the source form into `estimate` itself, as a string or an object per unit. Every consumer that reads `estimate.<unit>` as a number would break.
- Leave the round trip lossy and document it. A task moved between projects would freeze its fan-out without saying so.


== Out of scope

- Showing `estimate_counts` in the table output, which `mt eval` already prints.


== Dogfood

This task's own estimate is a `count(...)`, and moving it with `mt ls --json | mt import` keeps it.

