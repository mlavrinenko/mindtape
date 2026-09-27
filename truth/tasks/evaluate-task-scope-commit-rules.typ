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
  estimate: (src-files: 3),
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

- 🟢 `scope = "task"` rules run per task the commit changes or names. `head(<expr>)`, `staged(<expr>)` and `worktree(<expr>)` evaluate any expression, graph predicates included, against that state of the project.
- 🟢 A bare task field is refused at load. `exists` is false where the file is absent; `into("s")` means `staged(status == "s") && !head(status == "s")`; `named` and commit fields are stateless.

```toml
[[commits.rules]]
name    = "flip-with-work"
scope   = "task"
when    = 'into("wip") || into("done")'
assert  = 'named && (!tasks_only || staged(has_tag("decision")))'
message = "start or close a task in a commit that carries its work"
```

- 🟢 `when = 'worktree(status) != staged(status)'` warns about a flip left unstaged.


== Changes

- `crates/mindtape-cli/tests/commit_msg_task_e2e.rs` (red first): a close-only commit refused; a close with work passes; an estimate added in the closing commit is seen through `staged(...)`; a bare field is refused at load.
- `crates/mindtape/src/commit_tasks.rs` (new): the project as of `HEAD`, the index and the working tree, as overlays of the changed files on one load.
- `crates/mindtape/src/filter_predicates.rs`: the three state wrappers, `exists`, `named`.
- `crates/mindtape/src/config_hooks.rs`: share `into()`'s desugaring.
- `www/content/config.typ`: the task scope and the state table.


== Alternatives

- One implicit state plus a look-back (`before(...)`): the reader has to know which state a bare field means; `HEAD` reads a field edited in the same commit stale, the staged state needs `from("s")` hacks for history.
- A default state for bare fields: shorter to write, but a rule is written once and read many times, and every reader would have to know which state a bare field means.
- Dotted prefixes (`head.status`): cover fields but not predicates like `terminal`, `ready` or `has(f)`.


== Out of scope

- `worktree(...)` in `mt check --commits`: a commit range has no working tree, so a rule reading it is skipped there and reported as skipped.

- Commit ranges: `truth/tasks/check-commit-rules-over-a-commit-range.typ`.


== Dogfood

`truth/tasks/move-this-repos-task-flips-into-work-commits.typ` turns on `flip-with-work` here.

