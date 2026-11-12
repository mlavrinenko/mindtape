#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Scope touches() answers to the state's task set",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.95,
    ease: 6,
  ),
  difficulty: 3,
  links: (
    (
      task: "make-commit-classification-use-the-states-config.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "gating", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a commit rule reads the same touches() answer whatever order it compares the states in],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[one cache cannot hand one state's task set to another],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 12),
)

== AS-IS

+ Reviewing the commit-state config series, where head(...) and staged(...) read the same Config git cache.
+ 🔴 #raw("crates/mindtape/src/commit/refs.rs"): the RefsCache answer key is `set.is_some()`, so `head(touches(...))` and `staged(touches(...))` share whichever answer was computed first even when their task sets differ.

```
let key = format!("{}\0{path}", set.is_some());
...
lock(&self.answers).insert(key, Arc::clone(&touched));
```

The resolved `Touched` depends on the set's stems and rename walk, so a task deleted in the index answers differently by state:

```
head(touches("src/x.rs")) -> true
staged(touches("src/x.rs")) -> false
```


== TO-BE

- 🟢 An answer is keyed by the task set it resolved against, so `head(touches(...))` and `staged(touches(...))` cannot share a result.
- 🟢 A rule's outcome is the same whichever state's wrapper comes first in the expression.

```
let key = answer_key(path, set);
```


== Changes

- `crates/mindtape/src/commit/refs.rs`: include a stable identity of the task set in the answer key.
- `crates/mindtape/src/commit/refs_tests.rs`: pin two different sets over one path, in both orders.


== Out of scope

- Replacing the working config that `TaskScope` hands `touches()`. That is the rename map's separate decision.
- The `head(...)`/`staged(...)` expression syntax or rule semantics.


== Dogfood

A commit rule that compares `head(touches("src/x.rs"))` with `staged(touches("src/x.rs"))` gives the same verdict in either operand order.

