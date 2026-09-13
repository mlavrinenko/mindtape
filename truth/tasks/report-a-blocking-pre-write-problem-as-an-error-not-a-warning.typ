#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Report a blocking pre write problem as an error not a warning",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 3),
  assignee: "agent",
  status: done(
    2026,
    9,
    13,
  )[blocking pre-write problems report as error: and warn-severity breaches as warning:, pinned by mt add and mt recur],
)

== Summary

`crates/mindtape-cli/src/pre_write.rs:169` prints every blocking problem with
a `warning:` prefix, and since `[[tasks.rules]]` landed it prints genuine
non-blocking warnings with that same prefix two lines above. The two outcomes
are now indistinguishable at the line that reports them:

```
warning: regression-in-the-slug-dedup-path: rules.imperative-title: start the title with an imperative verb
Error: 1 validation issue(s) as filled; nothing was written
```

Only the trailing `Error:` line says which of the two happened, and a write
that lands prints no such line at all — so a reader scanning the per-issue
lines learns nothing. The prefix predates the rules work (`6a21382`); what
the rules work added is a second, real meaning for it.


== Scope

+ Prefix `verdict.problems` with `error:` and leave `verdict.warnings` at
  `warning:`, so the prefix carries the outcome the way `mt check` already
  makes it carry severity.
+ Check the same seam in `task_edit::report_issues`, which mirrors this shape
  on the post-write fallback path.
+ Cover it: an error-severity rule refusing an `mt add`, and a warn-severity
  one letting the same write land, asserting the two prefixes differ.

