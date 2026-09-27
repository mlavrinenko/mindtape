#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Evaluate task-scope commit rules",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("mt", "cli"),
  estimate: (files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[status flips ride with the work they claim, checked at commit time],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 27),
  links: depends-on("check-commit-messages-against-commit-rules.typ"),
)

== Problem

+ `git log --format=%s -300 | rg -c '^task: (start|close)'` in this repo.
+ 🔴 108 of the last 300 commits carry only a status flip: a close lands apart from its work, so reverting the work leaves the task done, and nothing can check it because commit rules see only the message.


== TO-BE

- 🟢 `scope = "task"` rules run once per task the commit changes or names. Fields read the staged task (the state the commit leaves), `before(<expr>)` evaluates an expression on the `HEAD` version, and `into("s")` is sugar for `status == "s" && !before(status == "s")`; plus `named`, `added`, `removed` and the commit fields.

```toml
[[commits.rules]]
name    = "flip-with-work"
scope   = "task"
when    = 'into("wip") || into("done")'
assert  = 'named && (!tasks_only || has_tag("decision"))'
message = "start or close a task in a commit that carries its work"
```

- 🟢 `before(...)` is false for an added task; a removed task reads its `HEAD` version.


== Changes

- `crates/mindtape-cli/tests/commit_msg_task_e2e.rs` (red first): a close-only commit refused, a close with work passes, an estimate added in the closing commit is seen.
- `crates/mindtape/src/commit_tasks.rs` (new): the changed and named tasks, each evaluated from the index and from `HEAD` blobs.
- `crates/mindtape/src/filter_predicates.rs`: `before()`, `named`, `added`, `removed`.
- `crates/mindtape/src/config_hooks.rs`: share `into()`'s desugaring.
- `www/content/config.typ`: the task scope.


== Alternatives

- Fields read `HEAD`, `into()` the staged status (the `[[on.flip]]` meaning): a field edited in the same commit is seen stale, so a rule requiring an estimate on close refuses the commit that adds it.
- Fields read staged, `from("s")` for the old status: "named only while open" needs a status name hardcoded.


== Out of scope

- Commit ranges: `truth/tasks/check-commit-rules-over-a-commit-range.typ`.


== Dogfood

`truth/tasks/move-this-repos-task-flips-into-work-commits.typ` turns on `flip-with-work` here.

