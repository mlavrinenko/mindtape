#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show shape errors in mt eval",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.98,
    ease: 9,
  ),
  difficulty: 1,
  links: (
    (
      task: "add-ast-grep-queries-as-a-shared-query-engine.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "cli", "stateless"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[the debug command and the gate tell the same story],
      ),
      ..(
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (docs-match-code: helps[the page and the command name the same failure]),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
)

== AS-IS

+ Verifying the fixes for the ast-grep review
+ 🔴 #raw("crates/mindtape-cli/src/cmd/eval.rs"): `mt eval` prints a plausible count while `mt check` reports the task's shape error.

A task with this estimate prints a manual-only count and no error:

```
count(1, ast_grep("rust", `warm($$$ARGS)`, 42))
```

```
$ mt eval num
estimate.files = 1
  1  manual
```

`mt check` on the same file says:

```
error: tasks/num.typ:5: estimate.files.queries[0]: glob must be a string
```

`crates/mindtape-cli/src/cmd/eval.rs` prints the breakdown and collector problems but never `task.shape_errors`.

== TO-BE

- 🟢 `mt eval` prints every shape error after the breakdown, so a dropped query cannot look like a valid manual count.
- 🟢 The message names the field and the cause, as `mt check` does.
- 🟢 A task with no shape error keeps its current output.

```
$ mt eval num
estimate.files = 1
  1  manual
error: estimate.files.queries[0]: glob must be a string
```

== Changes

- `crates/mindtape-cli/src/cmd/eval.rs`: print `task.shape_errors` after the breakdown.
- `crates/mindtape-cli/src/cmd/eval_tests.rs`: a red test for the shape-error text.
- `www/content/estimates.typ`: say a wrong-shaped query is reported by `mt check` and `mt eval`.

== Out of scope

- Changing `mt check`'s shape-error format.
- Turning the shape error into a fatal exit for `mt eval`.

== Dogfood

A wrong-shaped glob in a task this repo evaluates shows up in the debug command, not only in the gate.
